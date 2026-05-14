use crate::error::AppError;
use crate::services::utils::{
    get_project_root, is_workspace_member, normalize_project_name, refresh_metadata,
    register_in_workspace, run_command_silent,
};
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum CrateKind {
    /// A frontend client (`clients/`)
    Client,
    /// A WASM spin component (`components/`)
    Component,
    /// A standard library (`crates/`)
    Lib,
}

impl CrateKind {
    fn config(&self) -> (&'static str, &'static str) {
        match self {
            Self::Client => ("clients", "client"),
            Self::Component => ("components", "component"),
            Self::Lib => ("crates", "lib"),
        }
    }
}

pub(crate) fn add_package(name: &str, kind: CrateKind) -> Result<(), AppError> {
    check_cargo_generate_installed()?;

    if is_workspace_member(name)? {
        return Err(AppError::internal()
            .with_details(format!("Crate `{name}` is already a member of the workspace")));
    }

    let short_name = normalize_project_name(name);
    let full_name = format!("nx-{short_name}");
    let (sub_dir, template_name) = kind.config();
    let project_root = get_project_root()?;
    let destination = project_root.join(sub_dir);

    println!("> 🛠️ Generating {kind:?} `{full_name}` in `{sub_dir}/{short_name}`...");

    execute_generate(
        &short_name,
        &full_name,
        &destination,
        &format!("tooling/xtask/templates/{template_name}"),
    )?;

    if kind == CrateKind::Lib {
        register_in_workspace(&project_root, &full_name, &format!("{sub_dir}/{short_name}"))?;
    }

    refresh_metadata()?;

    println!("> ✅ Successfully created {kind:?} `{full_name}`");
    Ok(())
}

fn execute_generate(
    short_name: &str,
    full_name: &str,
    destination: &PathBuf,
    template_path: &str,
) -> Result<(), AppError> {
    let dest_str = destination
        .to_str()
        .ok_or_else(|| AppError::internal().with_details("Invalid UTF-8 path for destination"))?;

    let args = [
        "generate",
        "--path",
        template_path,
        "--name",
        short_name,
        "--destination",
        dest_str,
        "--define",
        &format!("name={full_name}"),
        "--define",
        &format!("shortname={short_name}"),
        "--silent",
    ];

    run_command_silent("cargo", &args).map_err(|e| {
        e.with_details(format!(
            "Failed to generate package using template `{template_path}`. \
             Make sure the template exists and cargo-generate is working."
        ))
    })
}

fn check_cargo_generate_installed() -> Result<(), AppError> {
    match std::process::Command::new("cargo")
        .arg("generate")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        _ => Err(AppError::internal()
            .with_message("`cargo-generate` is not installed.")
            .with_help("Install it by running: `cargo install cargo-generate`")),
    }
}
