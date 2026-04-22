//! # Workspace Linting Orchestrator
//!
//! Provides smart linting that applies the correct target architecture.
//! It automatically uses `wasm32-wasip2` for components but allows
//! manual target override for standard projects.

use crate::error::AppError;
use crate::services::utils::{
    BuildStrategy, detect_strategy, find_crate_in_metadata, get_packages_in_dir,
    resolve_crate_name, run_command,
};
use cargo_metadata::{Metadata, MetadataCommand};

pub(crate) fn lint_project(
    project: &str,
    target: Option<&String>,
    release: bool,
) -> Result<(), AppError> {
    let metadata = MetadataCommand::new()
        .exec()
        .map_err(|e| AppError::internal().with_message(format!("Failed to fetch metadata: {e}")))?;

    match project {
        "all" => {
            run_format_check()?;
            lint_group(&metadata, "components", Some(&"wasm32-wasip2".to_owned()), release)?;
            lint_group(&metadata, "clients", target, release)?;
        },
        "components" => {
            run_format_check()?;
            lint_group(&metadata, "components", Some(&"wasm32-wasip2".to_owned()), release)?;
        },
        "apps" | "clients" => {
            run_format_check()?;
            lint_group(&metadata, "clients", target, release)?;
        },
        name => {
            let name = resolve_crate_name(name)?;
            let pkg = find_crate_in_metadata(&metadata, &name)?;
            let strategy = detect_strategy(pkg);

            let effective_target = match strategy {
                BuildStrategy::Spin | BuildStrategy::Component => Some(&"wasm32-wasip2".to_owned()),
                BuildStrategy::Standard => target,
            };

            run_format_check()?;
            lint_single_crate(&pkg.name, effective_target, release)?;
        },
    }

    Ok(())
}

fn lint_group(
    metadata: &Metadata,
    folder_prefix: &str,
    target: Option<&String>,
    release: bool,
) -> Result<(), AppError> {
    let packages = get_packages_in_dir(metadata, folder_prefix);
    if packages.is_empty() {
        return Ok(());
    }

    println!("> 🔍 Linting group `{folder_prefix}`");
    for pkg in packages {
        lint_single_crate(&pkg.name, target, release)?;
    }
    Ok(())
}

fn lint_single_crate(name: &str, target: Option<&String>, release: bool) -> Result<(), AppError> {
    print!("> 🕵️  Clippy: {name:<20}");
    if let Some(t) = target {
        print!(" [target: {t}]");
    }
    println!();

    let mut args = vec!["clippy", "-p", name, "--all-targets", "--all-features"];

    if let Some(t) = target {
        args.extend(["--target", t.as_str()]);
    }

    if release {
        args.push("--release");
    }

    args.extend(["--", "-D", "warnings"]);
    run_command("cargo", &args)
}

fn run_format_check() -> Result<(), AppError> {
    println!("> 🧹  Formatting code...");
    run_command("cargo", &["fmt", "--all"])
    //run_command("cargo", &["fmt", "--all", "--", "--check"])
}
