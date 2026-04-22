#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub(crate) mod context;
pub mod error;
pub(crate) mod modules;
pub(crate) mod router;
pub(crate) mod services;
pub(crate) mod shared;

use crate::error::ConsoleError;
use crate::router::Route;
use crate::shared::utils::path::app_data_dir;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use nx_ui::prelude::*;

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

                Router::<Route> {}
            }
        }
    });

    Ok(())
}
