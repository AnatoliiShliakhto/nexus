use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn CenteredLayout() -> Element {
    rsx! {
        main {
            class: "flex flex-1 items-center justify-center min-h-0 min-w-0 overflow-hidden",
            Outlet::<Route> {}
        }
    }
}
