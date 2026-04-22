use super::store::{Notification, use_notifications};
use dioxus::prelude::*;
use dioxus_sdk_time::use_timeout;
use std::time::Duration;

#[component]
pub fn ToastRegion() -> Element {
    let store = use_notifications();

    let toasts = use_memo(move || {
        store.items.read().iter().filter(|n| n.show_toast).take(5).cloned().collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "toast-container",
            ol { class: "toast-list",
                for (index, notif) in toasts().into_iter().rev().enumerate() {
                    ToastItem {
                        key: "{notif.id}",
                        notif,
                        index,
                    }
                }
            }
        }
    }
}

#[component]
fn ToastItem(notif: Notification, index: usize) -> Element {
    let mut store = use_notifications();
    let id = notif.id.clone();

    let timeout = use_timeout(Duration::from_secs(notif.duration), move |()| {
        store.dismiss_toast(&id);
    });

    use_effect(move || {
        timeout.action(());
    });

    rsx! {
        li {
            class: "toast-item",
            style: "--depth: {index};",
            div {
                class: "toast",
                "data-type": notif.n_type.as_str(),

                div { class: "toast-content",
                    span { class: "toast-title", "{notif.title}" }
                    if let Some(desc) = &notif.description {
                        span { class: "toast-description", "{desc}" }
                    }
                }

                button {
                    class: "toast-close",
                    onclick: move |_| store.dismiss_toast(&notif.id),
                    i { class: "icon-[ph--x-bold]" }
                }
            }
        }
    }
}
