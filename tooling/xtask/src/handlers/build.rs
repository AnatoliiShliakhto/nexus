//! # Workspace Build Orchestrator
//!
//! This module coordinates the build processes for the entire workspace.
//! It intelligently detects the type of project (Spin component, WASI component, or standard binary)
//! and invokes the appropriate Cargo commands.

use crate::error::AppError;
use crate::services::utils::{
    BuildStrategy, artifact_filename, detect_strategy, find_crate_in_metadata, get_packages_in_dir,
    get_project_root, resolve_crate_name, resolve_target, run_command,
};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::fs;

/// Entry point for building projects in the workspace.
///
/// Handles building everything, specific logical groups, or individual crates.
pub(crate) fn build_project(
    project: &str,
    target: Option<&String>,
    release: bool,
) -> Result<(), AppError> {
    let metadata = MetadataCommand::new().exec().map_err(|e| {
        AppError::internal().with_message(format!("Failed to execute `cargo metadata`: {e}"))
    })?;

    let target = match target {
        Some(t) => t,
        None => &resolve_target()?,
    };

    match project {
        "all" => {
            build_group(&metadata, "components", target, release)?;
            build_group(&metadata, "clients", target, release)?;
        },
        "components" | "apps" => {
            let folder = if project == "apps" { "clients" } else { "components" };
            build_group(&metadata, folder, target, release)?;
        },
        name => {
            let name = resolve_crate_name(name)?;
            let pkg = find_crate_in_metadata(&metadata, &name)?;
            dispatch_build(pkg, target, release)?;
        },
    }

    Ok(())
}

/// Dispatches the build to the correct tool based on the crate's strategy.
fn dispatch_build(pkg: &Package, target: &str, release: bool) -> Result<(), AppError> {
    match detect_strategy(pkg) {
        BuildStrategy::Spin => build_spin(pkg, release)?,
        BuildStrategy::Component => build_component(pkg, release)?,
        BuildStrategy::Standard => build_standard(pkg, target, release)?,
    }
    Ok(())
}

/// Builds all crates located under a specific directory prefix within the workspace.
fn build_group(
    metadata: &Metadata,
    folder_prefix: &str,
    target: &str,
    release: bool,
) -> Result<(), AppError> {
    let packages = get_packages_in_dir(metadata, folder_prefix);

    if packages.is_empty() {
        println!("> ℹ️ No packages found in directory `{folder_prefix}`.");
        return Ok(());
    }

    for pkg in packages {
        dispatch_build(pkg, target, release)?;
    }

    Ok(())
}

// --- Tool-Specific Builders ---

/// Builds a Spin framework component and moves the artifact to the bin directory.
fn build_spin(pkg: &Package, release: bool) -> Result<(), AppError> {
    println!("> ⏳ Build spin component `{}`", pkg.name);

    let mut args = vec!["build", "-p", &pkg.name, "--target", "wasm32-wasip2"];
    if release {
        args.push("--release");
    }
    run_command("cargo", &args)?;

    copy_component_to_dist(pkg, release)?;
    println!("> ✅ Spin component `{}` built successfully!", pkg.name);

    Ok(())
}

/// Builds a pure WASI component using `cargo-component` and moves the artifact.
fn build_component(pkg: &Package, release: bool) -> Result<(), AppError> {
    println!("> ⏳ Build component `{}`", pkg.name);

    let mut args = vec!["component", "build", "-p", &pkg.name, "--target", "wasm32-wasip2"];
    if release {
        args.push("--release");
    }
    run_command("cargo", &args)?;

    copy_component_to_dist(pkg, release)?;
    println!("> ✅ Component `{}` built successfully!", pkg.name);

    Ok(())
}

/// Builds a standard Rust project (app/client) for the specified target.
fn build_standard(pkg: &Package, target: &str, release: bool) -> Result<(), AppError> {
    println!("> ⏳ Build project `{}`", pkg.name);

    let mut args = vec!["build", "-p", &pkg.name, "--target", target];
    if release {
        args.push("--release");
    }
    run_command("cargo", &args)?;

    println!("> ✅ Project `{}` built successfully!", pkg.name);
    Ok(())
}

// --- Helpers ---

/// Copies a compiled WASM component from the `target` directory into the server's `bin` layout.
fn copy_component_to_dist(pkg: &Package, release: bool) -> Result<(), AppError> {
    let root = get_project_root()?;
    let bin_name = artifact_filename(pkg, true);
    let dest_dir = root.join("dist").join("nx-server").join("components");

    // Convert underscores to dashes for the final bin destination
    let dest_file = dest_dir.join(bin_name.replace('_', "-"));

    let profile = if release { "release" } else { "debug" };
    let src_file = root.join("target").join("wasm32-wasip2").join(profile).join(&bin_name);

    if !src_file.exists() {
        return Err(AppError::internal()
            .with_message(format!("Component not found at: {}", src_file.display())));
    }

    fs::create_dir_all(&dest_dir)?;
    fs::copy(&src_file, &dest_file).map_err(|e| {
        AppError::internal().with_message(format!("Failed to copy to {}: {e}", dest_file.display()))
    })?;

    Ok(())
}
