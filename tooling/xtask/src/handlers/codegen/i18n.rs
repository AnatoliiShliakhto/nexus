use crate::error::{AppError, AppErrorExt};
use crate::services::utils::{get_packages_in_dir, get_project_root};
use cargo_metadata::MetadataCommand;
use fluent::FluentResource;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use unic_langid::LanguageIdentifier;

const GENERATED_DIR: &str = "crates/i18n/src/generated";
const BASE_LANGUAGE: &str = "en";
const REPORT_FILE_NAME: &str = "i18n-report.txt";

#[derive(Debug, Clone)]
struct TranslationEntry {
    key: String,
    module: String,
    value: String,
    source_file: PathBuf,
    line: usize,
}

#[derive(Debug, Clone)]
struct LanguageBundle {
    lang: String,
    entries: Vec<TranslationEntry>,
}

pub(crate) fn generate_i18n() -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let metadata = MetadataCommand::new()
        .exec()
        .map_err(|e| AppError::internal().with_message(format!("Metadata failed: {e}")))?;

    let crates = [
        get_packages_in_dir(&metadata, "crates"),
        get_packages_in_dir(&metadata, "components"),
        get_packages_in_dir(&metadata, "clients"),
        get_packages_in_dir(&metadata, "services"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut bundles: BTreeMap<String, Vec<TranslationEntry>> = BTreeMap::new();

    for pkg in crates {
        let crate_root = pkg
            .manifest_path
            .parent()
            .ok_or_else(|| {
                AppError::internal()
                    .with_details(format!("Missing parent path for crate `{}`", pkg.name))
            })?
            .as_std_path()
            .to_path_buf();

        let i18n_dir = crate_root.join("i18n");
        if !i18n_dir.exists() {
            continue;
        }

        let pkg_name = pkg.name.to_string().to_snake_case().replace("nx_", "");

        for entry in fs::read_dir(&i18n_dir)
            .with_details_fn(|| format!("Failed to read `{}`", i18n_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("ftl") {
                continue;
            }

            let lang = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    AppError::parse()
                        .with_details(format!("Invalid FTL filename: `{}`", path.display()))
                })?
                .to_owned();

            validate_lang_id(&lang)?;
            let entries = parse_ftl_file(&pkg_name, &path)?;
            bundles.entry(lang).or_default().extend(entries);
        }
    }

    if bundles.is_empty() {
        return Ok(());
    }

    let mut generated_bundles = Vec::new();

    for (lang, mut entries) in bundles {
        entries.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.source_file.cmp(&b.source_file)));
        let deduped = dedupe_entries(&lang, entries)?;
        generated_bundles.push(LanguageBundle { lang, entries: deduped });
    }

    generated_bundles.sort_by(|a, b| a.lang.cmp(&b.lang));

    let generated_dir = project_root.join(GENERATED_DIR);
    let bundles_dir = generated_dir.join("i18n");
    fs::create_dir_all(&bundles_dir)?;

    let base_bundle =
        generated_bundles.iter().find(|bundle| bundle.lang == BASE_LANGUAGE).ok_or_else(|| {
            AppError::codegen()
                .with_details(format!("Base language `{BASE_LANGUAGE}` bundle is missing"))
        })?;

    validate_bundle_consistency(&project_root, &generated_bundles, base_bundle)?;

    for bundle in &generated_bundles {
        let path = bundles_dir.join(format!("{}.ftl", bundle.lang));
        let content = render_ftl(&bundle.entries);
        fs::write(&path, content)
            .with_details_fn(|| format!("Failed to write `{}`", path.display()))?;
    }

    let mod_rs = render_mod_rs(&generated_bundles);
    let mod_path = generated_dir.join("mod.rs");
    fs::write(&mod_path, mod_rs)
        .with_details_fn(|| format!("Failed to write `{}`", mod_path.display()))?;

    println!("> ✅ Generated i18n bundles and localization module.");
    Ok(())
}

fn validate_lang_id(lang: &str) -> Result<(), AppError> {
    lang.parse::<LanguageIdentifier>().map_err(|e| {
        AppError::parse().with_details(format!("Invalid language id `{lang}`: {e}"))
    })?;
    Ok(())
}

fn parse_ftl_file(module: &str, path: &Path) -> Result<Vec<TranslationEntry>, AppError> {
    let raw = fs::read_to_string(path)
        .with_details_fn(|| format!("Failed to read `{}`", path.display()))?;

    FluentResource::try_new(raw.clone()).map_err(|(_res, errs)| {
        AppError::parse().with_details(format!("Invalid FTL in `{}`: {errs:?}", path.display()))
    })?;

    let mut entries = Vec::new();
    let mut current_key: Option<(String, usize)> = None;
    let mut current_value = String::new();

    let flush = |entries: &mut Vec<TranslationEntry>,
                 current_key: &mut Option<(String, usize)>,
                 current_value: &mut String,
                 source_file: &Path| {
        if let Some((key, line)) = current_key.take() {
            entries.push(TranslationEntry {
                key,
                module: module.to_owned(),
                value: current_value.trim_end().to_owned(),
                source_file: source_file.to_path_buf(),
                line,
            });
            current_value.clear();
        }
    };

    for (idx, line) in raw.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            flush(&mut entries, &mut current_key, &mut current_value, path);
            continue;
        }

        let is_new_message =
            !line.starts_with(' ') && !line.starts_with('\t') && trimmed.contains('=');

        if is_new_message {
            flush(&mut entries, &mut current_key, &mut current_value, path);

            let Some((key, value)) = trimmed.split_once('=') else {
                return Err(AppError::parse().with_details(format!(
                    "Invalid FTL line {line_no} in `{}`: `{line}`",
                    path.display(),
                )));
            };

            let key = key.trim();
            let value = value.trim();

            if key.is_empty() {
                return Err(AppError::parse().with_details(format!(
                    "Empty FTL key on line {line_no} in `{}`",
                    path.display()
                )));
            }

            current_key = Some((key.to_owned(), line_no));
            current_value.push_str(value);
        } else if current_key.is_some() {
            current_value.push('\n');
            current_value.push_str(line);
        } else {
            return Err(AppError::parse().with_details(format!(
                "Unexpected FTL content on line {line_no} in `{}`: `{line}`",
                path.display(),
            )));
        }
    }

    flush(&mut entries, &mut current_key, &mut current_value, path);

    Ok(entries)
}

fn dedupe_entries(
    lang: &str,
    entries: Vec<TranslationEntry>,
) -> Result<Vec<TranslationEntry>, AppError> {
    let mut seen: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();
    let mut unique = Vec::with_capacity(entries.len());

    for entry in entries {
        if let Some((prev_file, prev_line)) =
            seen.insert(entry.key.clone(), (entry.source_file.clone(), entry.line))
        {
            return Err(AppError::codegen().with_details(format!(
                "Duplicate i18n key `{}` for language `{lang}`\n  first:  {}:{}\n  second: {}:{}",
                entry.key,
                prev_file.display(),
                prev_line,
                entry.source_file.display(),
                entry.line
            )));
        }
        unique.push(entry);
    }

    Ok(unique)
}

fn validate_bundle_consistency(
    project_root: &Path,
    bundles: &[LanguageBundle],
    base_bundle: &LanguageBundle,
) -> Result<(), AppError> {
    let report_path = project_root.join("target").join("xtask").join(REPORT_FILE_NAME);

    let base_keys: BTreeMap<&str, &TranslationEntry> =
        base_bundle.entries.iter().map(|e| (e.key.as_str(), e)).collect();

    let mut report = String::new();
    let mut has_issues = false;

    report.push_str("I18n consistency report\n");
    let _ = write!(report, "Base language: {BASE_LANGUAGE}\n\n");

    for bundle in bundles.iter().filter(|b| b.lang != BASE_LANGUAGE) {
        let lang_keys: BTreeMap<&str, &TranslationEntry> =
            bundle.entries.iter().map(|e| (e.key.as_str(), e)).collect();

        let missing_in_lang: Vec<&&str> =
            base_keys.keys().filter(|key| !lang_keys.contains_key(**key)).collect();

        let extra_in_lang: Vec<&&str> =
            lang_keys.keys().filter(|key| !base_keys.contains_key(**key)).collect();

        if missing_in_lang.is_empty() && extra_in_lang.is_empty() {
            continue;
        }

        has_issues = true;
        let _ = writeln!(report, "Language: {}", bundle.lang);

        if !missing_in_lang.is_empty() {
            report.push_str("Missing keys:\n");
            for key in &missing_in_lang {
                let entry = base_keys.get(**key).expect("key from base_keys must exist");

                let _ = writeln!(
                    report,
                    "- {key} (from {}:{})",
                    entry.source_file.display(),
                    entry.line
                );
            }
        }

        if !extra_in_lang.is_empty() {
            report.push_str("Extra keys:\n");
            for key in &extra_in_lang {
                let entry = lang_keys.get(**key).expect("key from lang_keys must exist");
                let _ = writeln!(
                    report,
                    "- {key} (from {}:{})",
                    entry.source_file.display(),
                    entry.line
                );
            }
        }

        report.push('\n');
    }

    if !has_issues {
        return Ok(());
    }

    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, &report)
        .with_details_fn(|| format!("Failed to write `{}`", report_path.display()))?;

    let mut details = String::new();
    details.push_str("I18n bundle mismatch detected.\n");
    let _ = write!(details, "Detailed report written to `{}`\n\n", report_path.display());
    details.push_str(&report);

    Err(AppError::codegen().with_details(details))
}

fn render_ftl(entries: &[TranslationEntry]) -> String {
    let mut out = String::new();

    for entry in entries {
        out.push_str(&entry.key);
        out.push_str(" = ");
        out.push_str(&entry.value);
        out.push('\n');
    }

    out
}

fn render_mod_rs(bundles: &[LanguageBundle]) -> String {
    let mut out = String::new();

    out.push_str("pub(crate) const BUNDLES: &[(&str, &str)] = &[\n");
    for bundle in bundles {
        let _ =
            writeln!(out, "    (\"{}\", include_str!(\"i18n/{}.ftl\")),", bundle.lang, bundle.lang);
    }
    out.push_str("];\n\n");

    out.push_str("pub mod localization {\n");

    let mut groups: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for bundle in bundles {
        for entry in &bundle.entries {
            let const_name = entry.key.replace('-', "_").to_shouty_snake_case();

            groups
                .entry(entry.module.clone())
                .or_default()
                .entry(const_name)
                .or_insert_with(|| entry.key.clone());
        }
    }

    for (module_name, items) in groups {
        let _ = writeln!(out, "    pub mod {module_name} {{");

        for (const_name, key) in items {
            let _ = writeln!(out, "        pub const {const_name}: &str = \"{key}\";");
        }

        out.push_str("    }\n");
    }

    out.push_str("}\n");
    out
}
