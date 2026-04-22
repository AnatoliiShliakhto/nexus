use crate::error::{AppError, AppErrorExt};
use crate::services::utils::{find_crate_in_metadata, get_packages_in_dir, get_project_root};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use fxhash::{FxHashMap, FxHashSet};
use heck::{ToSnakeCase, ToTitleCase};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

const MANIFEST_PATH: &str = "tooling/ops/src/generated/migrations_manifest.rs";
const COMPONENTS_DIR: &str = "components";

// --- Models ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeKind {
    Bootstrap,
    Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceNode {
    key: String,
    name: String,
    description: Option<String>,
    config: ServiceConfig,
    files: Vec<PathBuf>,
    kind: NodeKind,
}

/// Service configuration parsed from `[package.metadata.service]`
/// `deny_unknown_fields` ensures typos in Cargo.toml are caught at build time.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceConfig {
    #[serde(default)]
    depends_on: FxHashSet<String>,
    #[serde(default)]
    route: Option<String>,
    #[serde(default)]
    access: ServiceAccess,
    #[serde(default)]
    spin: Option<ServiceSpinConfig>,
    #[serde(default)]
    system: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ServiceAccess {
    /// Publicly accessible via Spin HTTP trigger
    Public,
    /// Publicly accessible via Spin HTTP trigger and requires valid Authentication/Tokens
    Protected,
    /// No route; used for Redis triggers or direct component-to-component calls
    #[default]
    Inner,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceSpinConfig {
    #[serde(default)]
    allowed_outbound_hosts: FxHashSet<String>,
    #[serde(default)]
    variables: FxHashMap<String, String>,
    #[serde(default)]
    environment: FxHashMap<String, String>,
}

pub(crate) fn generate_migrations() -> Result<(), AppError> {
    let metadata = MetadataCommand::new().exec().map_err(|e| {
        AppError::codegen().with_message(format!("Cargo metadata resolution failed: {e}"))
    })?;

    let raw_nodes = discover_nx_nodes(&metadata)?;
    check_node_consistency(&raw_nodes)?;

    let sorted_nodes = resolve_execution_order(raw_nodes)?;

    render_migrations(&sorted_nodes)?;
    render_spin_config(&sorted_nodes)?;

    println!("> ✅ Generated Nexus manifest: {} nodes resolved successfully.", sorted_nodes.len());
    Ok(())
}

// --- Discovery ---

fn discover_nx_nodes(metadata: &Metadata) -> Result<Vec<ServiceNode>, AppError> {
    let mut nodes = Vec::new();

    // 1. Bootstrap (Engine)
    let boot_pkg = find_crate_in_metadata(metadata, "ops")?;
    if let Some(node) = process_package(boot_pkg, NodeKind::Bootstrap)? {
        nodes.push(node);
    }

    // 2. Components
    let component_packages = get_packages_in_dir(metadata, COMPONENTS_DIR);
    for pkg in component_packages {
        if let Some(node) = process_package(pkg, NodeKind::Component)? {
            nodes.push(node);
        }
    }

    Ok(nodes)
}

fn process_package(pkg: &Package, kind: NodeKind) -> Result<Option<ServiceNode>, AppError> {
    let crate_path = pkg
        .manifest_path
        .parent()
        .ok_or_else(|| {
            AppError::codegen().with_message(format!("Missing parent path for crate: {}", pkg.name))
        })?
        .as_std_path();

    let migrations_dir = crate_path.join("migrations");
    let files =
        if migrations_dir.exists() { read_surql_files(&migrations_dir)? } else { Vec::new() };

    if kind == NodeKind::Bootstrap {
        if files.is_empty() {
            return Err(AppError::codegen()
                .with_message("No migration files found for the bootstrap engine."));
        }
        return Ok(Some(ServiceNode {
            key: "engine".to_owned(),
            name: "Engine".to_owned(),
            description: Some("Database Engine Bootstrap".to_owned()),
            config: ServiceConfig { system: true, ..Default::default() },
            files,
            kind,
        }));
    }

    let config = match pkg.metadata.get("service") {
        Some(metadata_val) => serde_json::from_value::<ServiceConfig>(metadata_val.clone())
            .map_err(|e| {
                AppError::codegen()
                    .with_message(format!("Malformed [package.metadata.service] in `{}`", pkg.name))
                    .with_details(e.to_string())
            })?,
        None => ServiceConfig::default(),
    };

    let key = pkg.name.to_string();
    let name = key.replace("nx-", "").to_title_case();

    Ok(Some(ServiceNode { key, name, description: pkg.description.clone(), config, files, kind }))
}

// --- Validation ---

fn check_node_consistency(nodes: &[ServiceNode]) -> Result<(), AppError> {
    let mut routes = FxHashMap::default();

    for node in nodes {
        if node.kind == NodeKind::Bootstrap || node.config.access == ServiceAccess::Inner {
            continue;
        }

        let route = node.config.route.as_deref().unwrap_or("/").to_owned();

        if let Some(existing_node_key) = routes.insert(route.clone(), node.key.clone()) {
            return Err(AppError::codegen()
                .with_message("Routing conflict detected")
                .with_details(format!(
                    "Duplicate route `{route}` claimed by both `{existing_node_key}` and `{}`",
                    node.key
                )));
        }
    }
    Ok(())
}

// --- Execution Order Resolution (Kahn's Algorithm) ---

fn resolve_execution_order(nodes: Vec<ServiceNode>) -> Result<Vec<ServiceNode>, AppError> {
    let mut node_map: FxHashMap<String, ServiceNode> =
        nodes.into_iter().map(|n| (n.key.clone(), n)).collect();

    let boot_key = node_map
        .values()
        .find(|n| n.kind == NodeKind::Bootstrap)
        .map(|n| n.key.clone())
        .ok_or_else(|| {
            AppError::codegen()
                .with_details("Bootstrap node (migration) is missing from the graph.")
        })?;

    let mut adj: FxHashMap<String, Vec<String>> = FxHashMap::default();
    let mut in_degree: FxHashMap<String, usize> = node_map.keys().map(|k| (k.clone(), 0)).collect();

    // Build the graph
    for (key, node) in &node_map {
        let mut deps = node.config.depends_on.clone();

        // All components implicitly depend on the bootstrap engine
        if node.kind == NodeKind::Component {
            deps.insert(boot_key.clone());
        }

        for dep in deps {
            if !node_map.contains_key(&dep) {
                return Err(AppError::codegen().with_details(format!(
                    "Component `{key}` declares a dependency on an unknown component `{dep}`"
                )));
            }
            adj.entry(dep).or_default().push(key.clone());
            *in_degree.get_mut(key).expect("Key exists in in_degree map") += 1;
        }
    }

    // Priority queue for deterministic, typed sorting
    let mut queue = BinaryHeap::new();
    for (key, &deg) in &in_degree {
        if deg == 0 {
            queue.push(PriorityNode::new(key.clone(), node_map[key].kind));
        }
    }

    let mut sorted = Vec::with_capacity(node_map.len());

    while let Some(p_node) = queue.pop() {
        let key = p_node.key;

        if let Some(node) = node_map.remove(&key) {
            sorted.push(node);
        }

        if let Some(neighbors) = adj.get(&key) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).ok_or_else(|| {
                    AppError::codegen().with_details("Neighbor not found in in_degree map")
                })?;
                *deg -= 1;
                if *deg == 0 {
                    let neighbor_kind = node_map[neighbor].kind;
                    queue.push(PriorityNode::new(neighbor.clone(), neighbor_kind));
                }
            }
        }
    }

    if !node_map.is_empty() {
        return Err(AppError::codegen()
            .with_details("Circular dependency detected in the component graph."));
    }

    Ok(sorted)
}

// --- Rendering Migration Manifest ---

fn render_migrations(nodes: &[ServiceNode]) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let manifest_content = render_manifest(nodes, &project_root)?;
    let output_path = project_root.join(MANIFEST_PATH);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, manifest_content)
        .with_details_fn(|| format!("Failed to write manifest to `{}`", output_path.display()))?;

    Ok(())
}

fn render_manifest(nodes: &[ServiceNode], root: &Path) -> Result<String, AppError> {
    let mut w = String::new();
    writeln!(w, "//! @generated by `cargo codegen`.")?;
    writeln!(w, "//! WARNING: Any manual changes made to this file will be overwritten.\n")?;
    writeln!(w, "use crate::domain::models::Migration;\n")?;

    writeln!(w, "#[must_use]")?;
    writeln!(w, "pub(crate) fn builtin_migrations() -> Vec<Migration> {{")?;
    writeln!(w, "    vec![")?;

    for node in nodes {
        for file in &node.files {
            let version = extract_version(file);
            let rel_path = resolve_relative_path(file, root)?;
            let checksum = calculate_checksum(file)?;
            let bootstrap = node.kind == NodeKind::Bootstrap;

            writeln!(w, "        Migration {{")?;
            writeln!(w, "            component: \"{}\",", escape_str(&node.key))?;
            writeln!(w, "            component_name: \"{}\",", escape_str(&node.name))?;
            writeln!(
                w,
                "            component_description: {:?},",
                &node.description.as_deref().map(escape_str)
            )?;
            writeln!(w, "            version: \"{version}\",")?;
            writeln!(w, "            sql: include_str!(\"{rel_path}\"),")?;
            writeln!(w, "            checksum: \"{checksum}\",")?;
            writeln!(w, "            bootstrap: {bootstrap},")?;
            writeln!(w, "            system: {},", node.config.system)?;
            writeln!(
                w,
                "            protected: {},",
                node.config.access == ServiceAccess::Protected
            )?;
            if let Some(route) = &node.config.route {
                writeln!(w, "            route: Some(\"/{}\"),", route.trim_matches('/'))?;
            } else {
                writeln!(w, "            route: None,")?;
            }
            writeln!(w, "        }},")?;
        }
    }

    writeln!(w, "    ]")?;
    writeln!(w, "}}")?;
    Ok(w)
}

// --- Rendering Spin Config ---

fn render_spin_config(nodes: &[ServiceNode]) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let mut doc = load_spin_template(project_root.join("spin.template.toml"))?;

    for node in nodes.iter().filter(|n| n.kind == NodeKind::Component) {
        if node.config.access == ServiceAccess::Inner {
            let variable_key = format!("{}_url", node.key.to_snake_case());
            let default_host = format!("http://{}.spin.internal", node.key);

            add_variable_with_default(&mut doc, &variable_key, &default_host)?;
        }
        add_http_trigger(&mut doc, node)?;
        add_component_record(&mut doc, node)?;
    }

    let output_content = doc.to_string();
    fs::write(project_root.join("spin.toml"), output_content)
        .map_err(|e| AppError::codegen().with_details(format!("Failed to write spin.toml: {e}")))?;

    Ok(())
}

fn load_spin_template(path: impl AsRef<Path>) -> Result<DocumentMut, AppError> {
    let raw_content = fs::read_to_string(path).map_err(|e| {
        AppError::codegen().with_details(format!("Failed to open `spin.template.toml`: {e}"))
    })?;

    let clean_toml_content = raw_content
        .lines()
        .skip_while(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n");

    let header = r"# @generated
# =============================================================================
# ⚠️ DO NOT EDIT THIS FILE BY HAND ⚠️
# =============================================================================
# This spin.toml manifest is auto-generated by the Codegen Pipeline.
# Any manual changes will be overwritten on the next build.
#
# SOURCE OF TRUTH:
# To update routes, security policies, or outbound hosts, modify the
# `[package.metadata.service]` section in the respective component's Cargo.toml.
# =============================================================================
";

    let updated_toml_content = format!("{header}\n{clean_toml_content}");

    updated_toml_content.parse::<DocumentMut>().map_err(|e| {
        AppError::parse().with_details(format!("Failed to parse spin template TOML: {e}"))
    })
}

// --- TOML DOM Manipulation Helpers ---

fn add_variable_with_default(
    doc: &mut DocumentMut,
    key: &str,
    default_value: &str,
) -> Result<(), AppError> {
    let mut table = InlineTable::new();
    table.insert("default", Value::from(default_value));

    let variables =
        doc.entry("variables").or_insert(Item::Table(Table::new())).as_table_mut().ok_or_else(
            || {
                AppError::codegen()
                    .with_details_fn(|| format!("{key}: `variables` root key must be a table"))
            },
        )?;

    variables.insert(key, Item::Value(Value::InlineTable(table)));
    Ok(())
}

fn add_http_trigger(doc: &mut DocumentMut, node: &ServiceNode) -> Result<(), AppError> {
    let mut trigger = Table::new();
    trigger.insert("component", Item::Value(Value::from(node.key.as_str())));

    if node.config.access == ServiceAccess::Inner {
        let mut route_table = InlineTable::new();
        route_table.insert("private", Value::from(true));
        route_table.fmt();
        trigger.insert("route", Item::Value(Value::InlineTable(route_table)));
    } else {
        if let Some(route) = node.config.route.as_deref() {
            trigger.insert("route", Item::Value(Value::from(format!("/{}/...", route.trim_matches('/')))));
        } else {
            return Err(AppError::codegen()
                .with_details_fn(|| format!("{}: `route` path is missing", node.key)));
        }
    }

    let trigger_root = doc.entry("trigger").or_insert(Item::Table(Table::new()));
    let http_array = trigger_root
        .as_table_mut()
        .ok_or_else(|| {
            AppError::codegen()
                .with_details_fn(|| format!("{}: `trigger` must be a table", node.key))
        })?
        .entry("http")
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| {
            AppError::codegen().with_details_fn(|| {
                format!("{}: `trigger.http` must be an array of tables", node.key)
            })
        })?;

    http_array.push(trigger);
    Ok(())
}

fn add_component_record(doc: &mut DocumentMut, node: &ServiceNode) -> Result<(), AppError> {
    let mut comp_table = Table::new();
    comp_table.insert(
        "source",
        Item::Value(Value::from(format!("dist/nx-server/components/{}.wasm", node.key))),
    );

    if let Some(spin) = &node.config.spin {
        if !spin.allowed_outbound_hosts.is_empty() {
            let hosts_array: Array =
                spin.allowed_outbound_hosts.iter().map(|h| Value::from(h.as_str())).collect();
            comp_table.insert("allowed_outbound_hosts", Item::Value(Value::Array(hosts_array)));
        }

        if !spin.variables.is_empty() {
            let mut vars_inline = InlineTable::new();
            for (k, v) in &spin.variables {
                vars_inline.insert(k, Value::from(v.as_str()));
            }
            comp_table.insert("variables", Item::Value(Value::InlineTable(vars_inline)));
        }

        if !spin.environment.is_empty() {
            let mut env_inline = InlineTable::new();
            for (k, v) in &spin.environment {
                env_inline.insert(k, Value::from(v.as_str()));
            }
            comp_table.insert("environment", Item::Value(Value::InlineTable(env_inline)));
        }
    }

    let components_root = doc.entry("component").or_insert(Item::Table(Table::new()));
    components_root
        .as_table_mut()
        .ok_or_else(|| AppError::parse().with_message("Failed to get `component` root table"))?
        .insert(&node.key, Item::Table(comp_table));

    Ok(())
}

// --- General Helpers ---

fn read_surql_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|e| e.to_str()).map_or(false, |ext| {
                matches!(ext.to_lowercase().as_str(), "surql" | "sql" | "surrealql")
            })
        })
        .collect();

    files.sort();
    Ok(files)
}

fn extract_version(path: &Path) -> String {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("0000").to_owned()
}

fn calculate_checksum(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn resolve_relative_path(path: &Path, root: &Path) -> Result<String, AppError> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| AppError::internal().with_details("Path is outside project root"))?;

    let rel_str =
        rel.to_str().ok_or_else(|| AppError::internal().with_details("Invalid unicode in path"))?;

    // Assumes `MANIFEST_PATH` is nested 4 directories deep.
    // Example: root/tooling/migration/src/generated/ -> ../../../../
    Ok(format!("../../../../{}", rel_str.replace('\\', "/")))
}

fn escape_str(s: &str) -> String {
    s.replace('"', "\\\"")
}

// --- Priority Queue ---

#[derive(Debug, Eq, PartialEq)]
struct PriorityNode {
    score: u8,
    key: String,
}

impl PriorityNode {
    const fn new(key: String, kind: NodeKind) -> Self {
        let score = match kind {
            NodeKind::Bootstrap => 10,
            NodeKind::Component => 1,
        };
        Self { score, key }
    }
}

// Custom ordering to ensure Bootstrap nodes are popped first,
// and components are popped alphabetically for deterministic builds.
impl Ord for PriorityNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score).then_with(|| other.key.cmp(&self.key).reverse())
    }
}

impl PartialOrd for PriorityNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
