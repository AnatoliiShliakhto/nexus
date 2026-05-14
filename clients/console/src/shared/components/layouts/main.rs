use crate::router::Route;
use dioxus::prelude::*;
use nx_ui::prelude::*;
use nx_web_client::use_web_client;

#[component]
pub fn MainLayout() -> Element {
    let side = use_context_provider(|| Signal::new(SidebarSide::Left));
    let variant = use_context_provider(|| Signal::new(SidebarVariant::Sidebar));
    let collapsible = use_context_provider(|| Signal::new(SidebarCollapsible::Icon));

    let client = use_web_client();

    let logout = move |_| {
        let client = client.clone();
        spawn(async move {
            client.logout().await;
        });
    };

    rsx! {
        SidebarProvider {
            default_open: true,
            Sidebar {
                variant: variant(),
                collapsible: collapsible(),
                side: side(),

                SidebarHeader {
                    div { class: "flex items-center gap-2 overflow-hidden p-2 pb-3 border-b border-base-border-strong",
                        div {
                            class: "size-8 shrink-0 rounded bg-primary flex items-center justify-center text-primary-content font-bold shadow-md",
                            "N"
                        }

                        div {
                            class: "flex flex-col leading-none sidebar-header-text",

                            span { class: "font-semibold whitespace-nowrap text-sm", "Nexus console" }
                            span { class: "text-[10px] opacity-60 whitespace-nowrap", { env!("CARGO_PKG_VERSION") } }
                        }
                    }
                }

                SidebarContent {
                }

                SidebarFooter {
                    SidebarMenu {
                        SidebarMenuItem {
                            SidebarMenuButton {
                                onclick: logout,
                                i { class: "icon-[ph--user-circle-duotone]" }
                                span { "Anatolii Shliakhto" }
                            }
                        }
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
                    Outlet::<Route> {}
                }
            }
        }
    }
}
