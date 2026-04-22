use crate::error::AppError;
use crate::services::utils::{
    get_project_root, is_workspace_member, normalize_project_name, refresh_metadata,
    register_in_workspace,
};
use cargo_generate::{GenerateArgs, TemplatePath, generate};
use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum CrateKind {
    /// A frontend client (`clients/`)
    Client,
    /// A WASM spin component (`components/`)
    Component,
    /// A standard library (`crates/`)
    Lib,
}

pub(crate) fn add_package(name: &str, kind: CrateKind) -> Result<(), AppError> {
    if is_workspace_member(name)? {
        return Err(AppError::internal()
            .with_details(format!("Crate `{name}` is already a member of the workspace")));
    }

    let name = normalize_project_name(name);

    match kind {
        CrateKind::Client => add_client_pkg(&name)?,
        CrateKind::Component => add_component_pkg(&name)?,
        CrateKind::Lib => add_library_pkg(&name)?,
    }

    Ok(())
}

fn add_client_pkg(name: &str) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let define = vec![format!("name=nx-{name}"), format!("shortname={name}")];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("clients")),
        define,
        template_path: TemplatePath {
            path: Some("tooling/xtask/templates/client".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args).map_err(|e| {
        AppError::cargo_generate().with_details(format!("Failed to generate `client` package: {e}"))
    })?;
    refresh_metadata()?;

    println!("> ✅ Created client `nx-{name}` with package `clients/{name}`");
    Ok(())
}

fn add_component_pkg(name: &str) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let define = vec![format!("name=nx-{name}"), format!("shortname={name}")];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("components")),
        define,
        template_path: TemplatePath {
            path: Some("tooling/xtask/templates/component".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args).map_err(|e| {
        AppError::cargo_generate()
            .with_details(format!("Failed to generate `component` package: {e}"))
    })?;
    refresh_metadata()?;

    println!("> ✅ Created component `nx-{name}` with package `components/{name}`");
    Ok(())
}

fn add_library_pkg(name: &str) -> Result<(), AppError> {
    let project_root = get_project_root()?;
    let define = vec![format!("name=nx-{name}"), format!("shortname={name}")];

    let args = GenerateArgs {
        name: Some(name.to_owned()),
        destination: Some(project_root.join("crates")),
        define,
        template_path: TemplatePath {
            path: Some("tooling/xtask/templates/lib".to_owned()),
            ..Default::default()
        },
        silent: true,
        ..Default::default()
    };

    generate(args).map_err(|e| {
        AppError::cargo_generate()
            .with_details(format!("Failed to generate `library` package: {e}"))
    })?;
    register_in_workspace(&project_root, &format!("nx-{name}"), &format!("crates/{name}"))?;
    refresh_metadata()?;

    println!("> ✅ Created library `nx-{name}` with package `crates/{name}`");
    Ok(())
}
