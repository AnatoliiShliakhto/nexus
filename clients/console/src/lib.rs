#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub(crate) mod context;
pub mod error;
pub(crate) mod modules;
pub(crate) mod router;
pub(crate) mod services;
pub(crate) mod shared;

use crate::error::ConsoleError;
use crate::modules::auth::views::LoginPage;
use crate::router::Route;
use crate::shared::utils::path::app_data_dir;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use heck::ToKebabCase;
use nx_error::ErrorMetadata;
use nx_ui::prelude::*;
use nx_web_client::session::SessionStatus;
use nx_web_client::use_session;
use std::borrow::Cow;

#[allow(clippy::volatile_composites)]
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// Run the console application.
///
/// # Errors
///
pub fn run() -> Result<(), ConsoleError> {
    let data_dir = app_data_dir();

    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_transparent(false)
        .with_always_on_top(false)
        .with_decorations(false)
        .with_content_protection(true)
        .with_min_inner_size(LogicalSize::new(800, 600));

    let launch_builder_config = Config::new()
        .with_disable_context_menu(!cfg!(debug_assertions))
        .with_window(window)
        .with_data_directory(data_dir)
        .with_custom_head(format!(
            r#"<link rel="stylesheet" href="{}">"#,
            MAIN_CSS.resolve().display()
        ))
        .with_menu(None);

    LaunchBuilder::new().with_cfg(launch_builder_config).launch(|| {
        let client_err = use_hook(|| {
            if let Err(e) =
                nx_web_client::use_init_web_client("nx-console", "http://localhost:3001")
            {
                tracing::error!(
                    target: "console::init",
                    error = %e.message(),
                    code = %e.code(),
                    details = %e.details().unwrap_or_default(),
                    "Failed to initialize web client"
                );
                Some(e.code())
            } else {
                info!(target: "console::init", "Web client initialized successfully");
                None
            }
        });

        let session = use_session();

        rsx! {
            nx_ui::Resources {}

            DesktopWindow {
                name: "console",
                title: "Nexus Console",
                icon: rsx! {
                    i {
                        class: "icon-[ph--cpu-duotone] text-primary size-4!",
                    }
                },
                if let Some(err) = client_err {
                    CriticalError { code: err }
                } else {
                    match &session.read().status {
                        SessionStatus::Active => rsx! { Router::<Route> {} },
                        SessionStatus::Pending => rsx! {},
                        _ => rsx! { LoginPage {} },
                    }
                }
            }
        }
    });

    Ok(())
}

#[component]
fn CriticalError(code: Cow<'static, str>) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-screen bg-background p-8 text-center",

            i { class: "icon-[ph--warning-duotone] text-danger size-16 mb-6" }

            h1 {
                class: "text-2xl font-bold text-white mb-2",
                "Initialization Failed"
            }

            p {
                class: "text-muted-foreground max-w-md mb-8",
                { t!(code.to_kebab_case()) }
            }

            button {
                class: "btn btn-primary",
                onclick: move |_| {

                },
                "Close App"
            }
        }
    }
}
