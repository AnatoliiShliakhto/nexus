use super::store::{Notification, NotificationType, use_init_notifications, use_notifications};
use super::toast::ToastRegion;
use crate::components::{Sheet, SheetContent, SheetHeader, SheetSide, SheetTitle, VirtualList};
use crate::t;
use dioxus::prelude::*;
use nx_i18n::tr::ui as tr;
use std::rc::Rc;

#[component]
pub fn NotificationCenter(children: Element) -> Element {
    let mut store = use_init_notifications();

    rsx! {
        { children }

        Sheet {
            open: (store.is_open)(),
            on_open_change: move |state| store.is_open.set(state),

            SheetContent {
                side: SheetSide::Right,

                SheetHeader {
                    div { class: "flex justify-between items-center w-full px-6 py-4 border-b border-base-border",
                        SheetTitle { { t!(tr::UI_NOTIFICATIONS_HEADER) } }
                        if !store.items.read().is_empty() {
                            button {
                                class: "text-xs font-medium text-primary hover:underline cursor-pointer",
                                onclick: move |_| store.clear_all(),
                                { t!(tr::UI_NOTIFICATIONS_CLEAR) }
                            }
                        }
                    }
                }

                div { class: "flex-1 overflow-hidden flex flex-col",
                    NotificationList { items: store.items }
                }
            }
        }
        ToastRegion {}
    }
}

#[component]
fn NotificationList(items: Signal<Vec<Notification>>) -> Element {
    let mut store = use_notifications();

    let remove_handler = move |id: Rc<str>| {
        store.items.write().retain(|n| n.id != id);
    };

    let list_len = use_memo(move || items.read().len());
    let is_empty = use_memo(move || list_len() == 0);

    if is_empty() {
        return rsx! {
            div { class: "flex-1 flex flex-col items-center justify-center gap-2 text-base-content/40",
                i { class: "icon-[ph--bell-slash-duotone] size-10" }
                p { class: "text-sm", { t!(tr::UI_NOTIFICATIONS_EMPTY,) } }
            }
        };
    }

    rsx! {
        ul { class: "notification-list h-full",
            VirtualList {
                count: list_len(),
                buffer: 5usize,
                estimate_size: move |_| 72,
                render_item: move |index| {
                    if list_len() == 0 { return rsx! {}; }
                    let reversed_index = list_len().saturating_sub(1).saturating_sub(index);
                    let item = items.peek().get(reversed_index).cloned();

                    if let Some(notif) = item {
                        rsx! {
                            MemoizedNotificationItem {
                                key: "{notif.id}",
                                notif: notif,
                                on_remove: remove_handler,
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        }
    }
}

#[component]
fn MemoizedNotificationItem(notif: Notification, on_remove: EventHandler<Rc<str>>) -> Element {
    rsx! { NotificationItem { notif, on_remove } }
}

#[component]
fn NotificationItem(notif: Notification, on_remove: EventHandler<Rc<str>>) -> Element {
    let kind_class = match notif.n_type {
        NotificationType::Success => "text-success icon-[ph--check-circle-fill]",
        NotificationType::Error => "text-error icon-[ph--warning-circle-fill]",
        NotificationType::Warning => "text-warning icon-[ph--warning-fill]",
        NotificationType::Info => "text-info icon-[ph--info-fill]",
    };

    rsx! {
        li {
            class: "notification-item group",
            button {
                class: "notification-remove-btn group-hover:opacity-100",
                onclick: move |e| {
                    e.stop_propagation();
                    on_remove.call(notif.id.clone());
                },
                i { class: "icon-[ph--x-bold] size-4" }
            }

            if !notif.is_read {
                div { class: "absolute left-0 top-4 bottom-4 w-1 bg-primary rounded-r-md" }
            }

            div { class: "flex items-start gap-4 pr-6",
                span { class: "mt-0.5 size-5 shrink-0 {kind_class}" }

                div { class: "flex flex-col gap-1 w-full",
                    span { class: "font-bold text-sm leading-tight text-base-content", "{notif.title}" }

                    if let Some(desc) = &notif.description {
                        p { class: "text-xs text-base-content/60 leading-relaxed", "{desc}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotificationBell(#[props(default = String::new())] class: String) -> Element {
    let mut store = use_notifications();
    let unread_count = use_memo(move || store.unread_count());

    rsx! {
        button {
            class: "relative flex items-center justify-center h-full w-10 transition-all duration-200",
            class: "btn btn-ghost border-none rounded-none text-base-content/70 hover:text-base-content",
            class: if unread_count() > 0 { "text-base-content" } else { "" },
            class: "{class}",

            onclick: move |_| {
                if *store.is_open.read() {
                    store.close();
                } else {
                    store.open();
                }
            },

            i {
                class: "size-4",
                class: if unread_count() > 0 { "icon-[ph--bell-fill] animate-wiggle" } else { "icon-[ph--bell-light]" }
            }

            if unread_count() > 0 {
                span {
                    class: "absolute top-2 right-2 flex items-center justify-center min-w-[12px]",
                    class: "h-[12px] px-0.5 text-[8px] font-black text-white bg-error",
                    class: "rounded-full ring-1 ring-base-100",
                    "{unread_count}"
                }
            }
        }
    }
}
