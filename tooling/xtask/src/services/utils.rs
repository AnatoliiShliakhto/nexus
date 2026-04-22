//! # Workspace Utilities
use crate::error::{AppError, AppErrorExt};
use cargo_metadata::{Metadata, MetadataCommand, Package, TargetKind};
use heck::ToKebabCase;
use serde::Deserialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml_edit::de;

#[derive(Debug, Deserialize)]
pub(crate) struct CrateInfo {
    #[serde(skip)]
    pub path: PathBuf,
    pub package: PackageInfo,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
}

/// Returns the absolute path to the root directory of the workspace.
pub(crate) fn get_project_root() -> Result<PathBuf, AppError> {
    let path = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));

    if !path.exists() {
        return Err(AppError::internal()
            .with_details_fn(|| format!("Workspace root not found at `{}`", path.display()))
            .with_help("The xtask crate must be in a sub-directory of the workspace root."));
    }

    Ok(path)
}

/// Discovers crates in a workspace subdirectory.
pub(crate) fn get_workspace_crates(sub_dir: impl AsRef<Path>) -> Result<Vec<CrateInfo>, AppError> {
    let root = get_project_root()?;
    let target_dir = root.join(sub_dir);

    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let mut crates = Vec::new();

    for entry in fs::read_dir(&target_dir)
        .with_details_fn(|| format!("Failed to read `{}`", target_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let cargo_path = path.join("Cargo.toml");

        if path.is_dir() && cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path)?;
            let mut info: CrateInfo = de::from_str(&content).map_err(|e| {
                AppError::parse()
                    .with_details(format!("Failed to parse {}: {e}", cargo_path.display()))
            })?;

            info.path = path;
            crates.push(info);
        }
    }

    crates.sort_by_key(|c| c.path.file_name().map(ToOwned::to_owned).unwrap_or_default());
    Ok(crates)
}

/// Prints a formatted console table of discovered crates.
pub(crate) fn render_crate_table(title: &str, crates: &[CrateInfo]) {
    println!("\n{title}:\n");
    println!("{:<15} {:<20} {:<45}", "Folder", "Crate Name", "Description");
    println!("{:-<80}", "");

    for info in crates {
        let folder = info.path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let desc = info.package.description.as_deref().unwrap_or("No description provided");
        println!("{:<15} {:<20} {:<45}", folder, info.package.name, desc);
    }
    println!();
}

/// Normalizes a project name to `kebab-case` and removes the ` nx-` suffix if present.
#[must_use]
pub(crate) fn normalize_project_name(project: &str) -> String {
    project.to_kebab_case().replace("nx-", "")
}

pub(crate) fn refresh_metadata() -> Result<(), AppError> {
    println!("> 🔄 Refreshing workspace metadata...");
    run_command_silent("cargo", &["metadata", "--format-version", "1"])
}

pub(crate) fn register_in_workspace(
    root: &Path,
    pkg_name: &str,
    pkg_path: &str,
) -> Result<(), AppError> {
    let toml_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&toml_path)?;

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| AppError::parse().with_details(format!("TOML error: {e}")))?;

    let pkg_version = doc["workspace"]["package"]["version"]
        .clone()
        .into_value()
        .map_err(|e| AppError::parse().with_details(format!("TOML error: {e}")))?;

    let deps = doc["workspace"]["dependencies"]
        .or_insert(toml_edit::table())
        .as_table_mut()
        .ok_or_else(|| AppError::parse().with_message("workspace.dependencies is not a table"))?;

    let mut inline = toml_edit::InlineTable::default();
    inline.insert("version", pkg_version);
    inline.insert("path", pkg_path.into());
    inline.insert("default-features", false.into());
    deps.insert(pkg_name, toml_edit::value(inline));
    deps.sort_values();

    fs::write(&toml_path, doc.to_string())?;
    Ok(())
}

pub(crate) fn run_command(cmd: &str, args: &[&str]) -> Result<(), AppError> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_details_fn(|| format!("Failed to execute `{cmd} {args:?}`"))?;

    if !status.success() {
        return Err(AppError::internal().with_details(format!("`{cmd}` failed: {status}")));
    }
    Ok(())
}

pub(crate) fn run_command_silent(cmd: &str, args: &[&str]) -> Result<(), AppError> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_details(format!("Failed to run `{cmd}`"))?;

    if !output.status.success() {
        let log_dir = get_project_root()?.join("target").join("xtask");
        fs::create_dir_all(&log_dir)?;

        let log_file = log_dir.join("errors.log");
        let mut file = OpenOptions::new().create(true).append(true).open(&log_file)?;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(
            file,
            "--- [{timestamp}] Error: {cmd} {args:?} ---\n{}\n",
            String::from_utf8_lossy(&output.stderr)
        )?;

        return Err(AppError::internal()
            .with_details(format!("Command `{cmd}` failed. See `{}`", log_file.display())));
    }
    Ok(())
}

pub(crate) fn is_workspace_member(name: &str) -> Result<bool, AppError> {
    let shortname = normalize_project_name(name);
    let name = format!("nx-{shortname}");

    let metadata = MetadataCommand::new()
        .exec()
        .map_err(|e| AppError::internal().with_message(format!("Metadata failed: {e}")))?;

    Ok(metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .any(|pkg| pkg.name == name || pkg.name == shortname))
}

pub(crate) fn resolve_crate_name(name: &str) -> Result<String, AppError> {
    let metadata = MetadataCommand::new()
        .exec()
        .map_err(|e| AppError::internal().with_message(format!("Metadata failed: {e}")))?;

    if let Some(pkg) = metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .find(|pkg| pkg.name == name)
    {
        return Ok(pkg.name.to_string());
    }

    let candidates: Vec<&str> = metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .filter(|pkg| pkg.name.contains(name))
        .map(|pkg| pkg.name.as_str())
        .collect();

    match candidates.len() {
        0 => Err(AppError::internal().with_details_fn(|| format!("Crate `{name}` not found"))),
        1 => Ok(candidates[0].to_owned()),
        _ => Err(AppError::internal()
            .with_details_fn(|| format!("Ambiguous name `{name}`. Matches: {candidates:?}"))
            .with_help("Try using exact crate name or specify a more specific search term")),
    }
}

pub(crate) fn resolve_target() -> Result<String, AppError> {
    let output = Command::new("rustc").arg("-vV").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .find_map(|l| l.strip_prefix("host: "))
        .map(|s| s.trim().to_owned())
        .ok_or_else(|| AppError::internal().with_message("Host triple not found"))
}

pub(crate) fn artifact_filename(pkg: &Package, is_wasm: bool) -> String {
    let target = pkg
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| matches!(k, TargetKind::Bin | TargetKind::CDyLib)))
        .unwrap_or(&pkg.targets[0]);

    let mut name = target.name.clone();

    if target.kind.iter().any(|k| matches!(&k, TargetKind::Lib | TargetKind::CDyLib)) {
        name = name.replace('-', "_");
    }

    match (is_wasm, cfg!(windows)) {
        (true, _) => format!("{name}.wasm"),
        (false, true) => format!("{name}.exe"),
        _ => name,
    }
}

pub(crate) fn get_packages_in_dir<'a>(
    metadata: &'a Metadata,
    dir_prefix: &str,
) -> Vec<&'a Package> {
    metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .filter(|pkg| {
            pkg.manifest_path
                .strip_prefix(&metadata.workspace_root)
                .map(|rel| rel.starts_with(dir_prefix))
                .unwrap_or(false)
        })
        .collect()
}

pub(crate) fn find_crate_in_metadata<'a>(
    metadata: &'a Metadata,
    name: &str,
) -> Result<&'a Package, AppError> {
    metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == name)
        .ok_or_else(|| AppError::internal().with_details(format!("Crate '{name}' not found")))
}

pub(crate) fn is_dep_feature_enabled(pkg: &Package, dependency: &str, feature: &str) -> bool {
    let dep = pkg.dependencies.iter().find(|d| d.name == dependency);

    dep.map_or(false, |d| d.features.iter().any(|f| f == feature))
}

pub(crate) enum BuildStrategy {
    Spin,
    Component,
    Standard,
}

pub(crate) fn detect_strategy(pkg: &Package) -> BuildStrategy {
    let has_spin = pkg
        .dependencies
        .iter()
        .any(|d| d.name == "nx-http" && d.features.iter().any(|f| f == "spin"));

    if has_spin {
        BuildStrategy::Spin
    } else if pkg.metadata.get("component").is_some() {
        BuildStrategy::Component
    } else {
        BuildStrategy::Standard
    }
}
