use crate::components::ResizeHandle;
use crate::enclose;
use crate::hooks::{StorageProvider, use_storage};
use crate::widgets::{NotificationBell, NotificationCenter};
use dioxus::desktop::{LogicalPosition, LogicalSize, use_window};
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Props, Clone, PartialEq, Serialize, Deserialize)]
struct WindowConfig {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { left: 100, top: 50, width: 800, height: 600, maximized: false }
    }
}

#[component]
pub fn DesktopWindow(
    #[props(into, default = Cow::Borrowed("window"))] name: Cow<'static, str>,
    #[props] icon: Option<Element>,
    #[props(default = ReadSignal::new(Signal::new(String::new())))] title: ReadSignal<String>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(button { class: "flex flex-1 min-h-0 min-w-0 overflow-hidden" });
    let merged = merge_attributes(vec![base, attributes]);

    let theme = use_storage("theme", || "light".to_owned());

    use_effect(move || {
        eval(&format!("document.documentElement.setAttribute('data-theme', '{theme}')"));
    });

    rsx! {
        div {
            class: "flex flex-col h-screen w-screen bg-base-200 text-base-content overflow-hidden",
            style: "--canvas-top: 2.25rem;",
            oncontextmenu: move |evt| {
                if !cfg!(debug_assertions) {
                    evt.prevent_default();
                    evt.stop_propagation();
                }
            },
            NotificationCenter {
                TitleBar { theme, name, icon, title },

                main {
                    ..merged,
                    { children }
                }
            }
            ResizeHandle {}
        }
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#[component]
fn TitleBar(
    theme: Signal<String>,
    name: Cow<'static, str>,
    icon: Option<Element>,
    title: ReadSignal<String>,
) -> Element {
    let window = use_window();
    let state = StorageProvider::<WindowConfig>::new(name, WindowConfig::default);
    let mut maximized = use_signal(|| false);

    use_hook(enclose!(window, state => move || {
        let config = state.read();
        window.set_outer_position(LogicalPosition::new(config.left, config.top));
        window.set_inner_size(LogicalSize::new(config.width, config.height));
        window.set_maximized(config.maximized);
        maximized.set(config.maximized);
    }));
    use_effect(enclose!(window => move || window.set_title(&title.read())));

    let minimize = enclose!(window => move |_| window.set_minimized(true));
    let toggle_maximized = enclose!(window, state => move |_| {
        maximized.set(!window.is_maximized());
        state.with_mut(move |config| {
            config.maximized = maximized();
        });
        window.set_maximized(maximized());
    });
    let close = enclose!(window, state => move |_| {
        if window.is_maximized() {
            window.close();
            return;
        }

        state.with_mut(|config| {
            config.maximized = window.is_maximized();

            if config.maximized {
                return;
            }

            let scale = window.scale_factor();

            if let Ok(pos) = window.outer_position() {
                config.left = (f64::from(pos.x) / scale) as u16;
                config.top = (f64::from(pos.y) / scale) as u16;
            }

            let size = window.inner_size();
            config.width = (f64::from(size.width) / scale) as u16;
            config.height = (f64::from(size.height) / scale) as u16;
        });
        window.close();
    });
    let on_drag = enclose!(window => move |_| window.drag());

    rsx! {
        div {
            class: "flex flex-row h-9 items-center justify-between bg-base-100 border-b border-strong z-50 shrink-0",
            onmousedown: on_drag,

            div {
                class: "flex items-center gap-3 px-4 shrink-0 min-w-0",
                if let Some(icon) = icon {
                    div {
                        class: "opacity-50 flex items-center justify-center shrink-0",
                        { icon }
                    }
                }
                span {
                    class: "text-[11px] font-bold tracking-widest uppercase truncate text-base-content/80",
                    "{title}"
                }
            }

            div {
                class: "flex items-center h-full shrink-0 ml-auto",
                onmousedown: move |evt| evt.stop_propagation(),
                NotificationBell {}
                if cfg!(debug_assertions) {
                    button {
                        class: "btn btn-ghost h-full w-10 rounded-none border-none hover:bg-warning/40",
                        onclick: move |_| window.devtool(),
                        i { class: "icon-[ph--bug-light]" }
                    }
                }
                ThemeToggle { theme },
                button {
                    class: "btn btn-ghost h-full w-10 rounded-none border-none",
                    onclick: minimize,
                    i { class: "icon-[ph--minus-light]" }
                }
                button {
                    class: "btn btn-ghost h-full w-10 rounded-none border-none",
                    onclick: toggle_maximized,
                    if maximized() {
                        i {
                            class: "icon-[ph--corners-in-light]"
                        }
                    } else {
                        i {
                            class: "icon-[ph--corners-out-light]"
                        }
                    }
                }
                button {
                    class: "btn btn-ghost h-full w-10 rounded-none hover:bg-error hover:text-white transition-colors border-none",
                    onclick: close,
                    i {
                        class: "icon-[ph--x-light]"
                    }
                }
            }
        }
    }
}

#[component]
fn ThemeToggle(mut theme: Signal<String>) -> Element {
    let is_dark = use_memo(move || &*theme.read() == "dark");

    rsx! {
        button {
            class: "btn btn-ghost h-full w-10 rounded-none border-none",
            onclick: move |_| {
                theme.set(if is_dark() { "light".into() } else { "dark".into() });
            },
            span {
                class: if is_dark() { "icon-[ph--moon-light]" } else { "icon-[ph--sun-light]" },
                class: "size-4"
            }
        }
    }
}
