#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttons;
mod controls;
mod dialogs;
mod login;
mod notifications;
mod scroll_area;
mod sheet;
mod sidebar;
mod text_area;

use crate::buttons::ButtonsDemo;
use crate::controls::ControlsDemo;
use crate::dialogs::DialogDemo;
use crate::login::LoginFormDemo;
use crate::notifications::NotificationsDemo;
use crate::scroll_area::ScrollAreaDemo;
use crate::sheet::SheetDemo;
use crate::sidebar::SidebarDemo;
use crate::text_area::TextAreaDemo;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use nx_ui::prelude::*;
use std::path::PathBuf;

#[allow(clippy::volatile_composites)]
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
        .join(env!("CARGO_PKG_NAME"));

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).ok();
    }

    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_transparent(false)
        .with_always_on_top(false)
        .with_decorations(false)
        .with_content_protection(false)
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
                name: "showcase",
                title: "Nexus UI Showcase",
                icon: rsx! {
                    i {
                        class: "icon-[ph--flask-duotone] text-primary size-4!",
                    }
                },

                Demo {}
            }
        }
    });
}

#[component]
fn MainContent() -> Element {
    rsx! {
        SidebarDemo {}
        LoginFormDemo {}
        ButtonsDemo {}
        ControlsDemo {}
        SheetDemo {}
        NotificationsDemo {}
        DialogDemo {}
        TextAreaDemo {}
        ScrollAreaDemo {}
    }
}

#[component]
pub fn Demo() -> Element {
    let side = use_context_provider(|| Signal::new(SidebarSide::Left));
    let variant = use_context_provider(|| Signal::new(SidebarVariant::Sidebar));
    let collapsible = use_context_provider(|| Signal::new(SidebarCollapsible::Icon));

    rsx! {
        SidebarProvider {
            default_open: true,
            Sidebar {
                variant: variant(),
                collapsible: collapsible(),
                side: side(),

                SidebarHeader {
                    div { class: "flex items-center gap-2 overflow-hidden p-2 pb-4 border-b border-base-border-strong",
                        div {
                            class: "size-8 shrink-0 rounded bg-primary flex items-center justify-center text-primary-content font-bold shadow-md",
                            "U"
                        }

                        div {
                            class: "flex flex-col leading-none sidebar-header-text",

                            span { class: "font-semibold whitespace-nowrap text-sm", "Nexus UI Showcase" }
                            span { class: "text-[10px] opacity-60 whitespace-nowrap", { env!("CARGO_PKG_VERSION") } }
                        }
                    }
                }

                SidebarContent {
                    SidebarGroup {
                        SidebarGroupLabel { "Core Services" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        is_active: true,
                                        i { class: "icon-[ph--house-line-duotone]" }
                                        span { "Overview" }
                                    }
                                }

                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        i { class: "icon-[ph--hard-drives-duotone]" }
                                        span { "Infrastructure" }
                                    }
                                    SidebarMenuSub {
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Nodes Cluster" } }
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Wasm Workers" } }
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Storage Pools" } }
                                    }
                                }

                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        i { class: "icon-[ph--shield-check-duotone]" }
                                        span { "Security" }
                                    }
                                    SidebarMenuSub {
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Identity (DPoP)" } }
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Access Policies" } }
                                        SidebarMenuSubItem { SidebarMenuSubButton { "Audit Logs" } }
                                    }
                                }
                            }
                        }
                    }

                    SidebarGroup {
                        SidebarGroupLabel { "Monitoring" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        i { class: "icon-[ph--chart-line-up-duotone]" }
                                        span { "Performance" }
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        i { class: "icon-[ph--activity-duotone]" }
                                        span { "Network Traffic" }
                                    }
                                }
                            }
                        }
                    }
                }

                SidebarFooter {
                    SidebarMenu {
                        SidebarMenuItem {
                            SidebarMenuButton {
                                // tooltip: rsx! { "User Profile" },
                                i { class: "icon-[ph--user-circle-duotone]" }
                                span { "Anatolii Shliakhto" }
                            }
                        }
                        // SidebarMenuItem {
                        //     SidebarMenuButton {
                        //         i { class: "icon-[ph--gear-duotone]" }
                        //         span { "Settings" }
                        //     }
                        // }
                    }
                }

                SidebarRail {}
            }

            SidebarInset {
                header {
                    class: "flex h-14 shrink-0 items-center gap-2 border-b border-base-border px-4 bg-base-100",
                    SidebarTrigger {}
                    SidebarSeparator { horizontal: false }
                    nav { class: "flex items-center gap-2 text-sm",
                        span { class: "text-base-content/50", "Infrastructure" }
                        i { class: "icon-[ph--caret-right] size-3 opacity-30" }
                        span { class: "font-medium", "Nodes Cluster" }
                    }
                }
                div {
                    class: "flex flex-1 flex-col min-h-0 p-6 gap-5 overflow-y-auto bg-base-100",
                    MainContent {}
                }
            }
        }
    }
}
