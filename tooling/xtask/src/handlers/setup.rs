use crate::error::{AppError, AppErrorExt};
use crate::handlers::keys::rotate_keys;
use crate::services::utils::run_command_silent;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs, io};

/// Tools required for development
const REQUIRED_TOOLS: &[(&str, &str)] = &[
    ("cargo-generate", "cargo-generate"),
    ("cargo-component", "cargo-component"),
    ("cargo-audit", "cargo-audit"),
    ("cargo-deny", "cargo-deny"),
    ("cargo-nextest", "cargo-nextest"),
    ("sccache", "sccache"),
    ("dx", "dioxus-cli"),
];

/// Targets required for cross-platform/WASM development
const REQUIRED_TARGETS: &[&str] = &["wasm32-wasip2", "wasm32-unknown-unknown"];

/// Set up the development environment for `Nexus`.
///
/// # Result
/// Returns `Ok(())` after installing required tools, targets, and generating a keyset.
///
/// # Errors
/// Returns an error if tool installation fails, required targets cannot be added,
/// or keyset generation/writes fail.
pub(crate) fn setup_project() -> Result<(), AppError> {
    println!("> Starting `Nexus` development setup...");

    setup_env()?;
    check_node_environment()?;
    check_spin();

    install_dependencies()?;

    rotate_keys()?;

    println!("\n> ✨ Setup complete! You are ready to develop for `Nexus`.");

    Ok(())
}

fn install_dependencies() -> Result<(), AppError> {
    for (bin, package) in REQUIRED_TOOLS {
        if is_tool_installed(bin) {
            print!("> ⏳ `{bin}` is already installed. Trying update...\t");
        } else {
            print!("> ⏳ Installing `{package}`...\t");
        }
        io::stdout().flush()?;
        match run_command_silent("cargo", &["install", package, "--locked"]) {
            Ok(()) => println!("✅"),
            Err(e) => println!("❌\n\t{e}"),
        }
    }

    let installed_targets_output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .with_details("Failed to list installed rustup targets")?;

    let installed_targets = String::from_utf8_lossy(&installed_targets_output.stdout);

    for target in REQUIRED_TARGETS {
        if installed_targets.contains(target) {
            println!("> ✅ Target `{target}` is already installed.");
            continue;
        }

        print!("> 🦀 Adding rustup target `{target}`...\t");
        io::stdout().flush()?;
        match run_command_silent("rustup", &["target", "add", target]) {
            Ok(()) => println!("✅"),
            Err(e) => println!("❌\n\t{e}"),
        }
    }

    Ok(())
}

fn setup_env() -> Result<(), AppError> {
    let example_env = ".env-example";
    let target_env = ".env";

    if Path::new(target_env).exists() {
        println!("> ✅ `{target_env}` already exists. Skipping.");
        return Ok(());
    }

    if !Path::new(example_env).exists() {
        println!("> ❌ `{example_env}` not found. Could not create `{target_env}`.");
        return Ok(());
    }

    print!("> 📄 Creating `{target_env}` from `{example_env}`...\t");
    io::stdout().flush()?;

    match fs::copy(example_env, target_env) {
        Ok(_) => {
            println!("✅");
            Ok(())
        },
        Err(e) => {
            println!("❌");
            Err(AppError::from(e)
                .with_details(format!("Failed to copy {example_env} to {target_env}")))
        },
    }
}

fn is_tool_installed(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn check_node_environment() -> Result<(), AppError> {
    if is_tool_installed("node") {
        println!("> ✅ Node.js is installed.");

        if is_tool_installed("ncu") {
            println!("> ✅ ncu (npm-check-updates) is installed.");
        } else {
            print!("> ⏳ Installing ncu globally via npm...\t");
            io::stdout().flush()?;
            let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };
            match run_command_silent(npm_cmd, &["install", "-g", "npm-check-updates"]) {
                Ok(()) => println!("✅"),
                Err(e) => println!("❌\n\t{e}"),
            }
        }
    } else {
        warn_missing_node();
    }
    Ok(())
}

fn warn_missing_node() {
    println!("> ❌ Node.js not found!");
    println!("\tNode.js is required for asset management and frontend builds.");
    println!("\tPlease download it from: https://nodejs.org/");
    println!("\tAfter installing, restart your terminal and run `cargo xtask setup` again.\n");
}

fn check_spin() {
    if is_tool_installed("spin") {
        println!("> ✅ Fermyon Spin is already installed.");
        return;
    }
    println!("> ❌ Fermyon Spin not found!");
    println!("\tFermyon Spin is required for asset management and frontend builds.");
    println!("\tPlease download it from: https://github.com/spinframework/spin/releases/latest/");
    println!("\tAfter installing, restart your terminal and run `cargo xtask setup` again.\n");
}
