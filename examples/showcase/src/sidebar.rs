use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn SidebarDemo() -> Element {
    let sidebar = use_sidebar();
    let mut side = use_context::<Signal<SidebarSide>>();
    let mut variant = use_context::<Signal<SidebarVariant>>();
    let mut collapsible = use_context::<Signal<SidebarCollapsible>>();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Sidebar" }
            }
            CardContent {
                class: "card-content flex flex-col gap-2",
                div {
                    class: "flex gap-2",
                    Button {
                        onclick: move |_| sidebar.toggle(),
                        i { class: "icon-[ph--sidebar]" }
                    }
                    Separator { horizontal: false }
                    Button {
                        onclick: move |_| side.set(SidebarSide::Left),
                        "Left"
                    }
                    Button {
                        onclick: move |_| side.set(SidebarSide::Right),
                        "Right"
                    }
                }
                div {
                    class: "flex gap-2",
                    Button {
                        onclick: move |_| variant.set(SidebarVariant::Sidebar),
                        "Sidebar"
                    }
                    Button {
                        onclick: move |_| variant.set(SidebarVariant::Floating),
                        "Floating"
                    }
                    Button {
                        onclick: move |_| variant.set(SidebarVariant::Inset),
                        "Inset"
                    }
                    Separator { horizontal: false }
                    Button {
                        onclick: move |_| collapsible.set(SidebarCollapsible::None),
                        "None"
                    }
                    Button {
                        onclick: move |_| collapsible.set(SidebarCollapsible::Icon),
                        "Icon"
                    }
                    Button {
                        onclick: move |_| collapsible.set(SidebarCollapsible::Offcanvas),
                        "Offcanvas"
                    }
                }
            }
        }
    }
}
