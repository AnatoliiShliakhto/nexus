use dioxus::prelude::*;
use nx_ui::macros::i18n::set_locale;
use nx_ui::prelude::{tr::console as tr, *};

#[component]
pub(crate) fn LoginPage() -> Element {
    rsx! {
        div { class: "flex flex-col gap-6",

            div { class: "text-center space-y-2",
                i { class: "icon-[ph--shield-check-duotone] text-5xl text-primary" }
                h1 { class: "text-2xl font-bold tracking-tight", { t!(tr::CON_LOGIN_TITLE) } }
                p { class: "text-base-content/60 text-sm", { t!(tr::CON_LOGIN_DESCRIPTION) } }
            }

            Card { class: "card shadow-lg w-[400px] border-t-4 border-t-primary",
                CardContent { class: "card-content gap-4 pt-6",
                    div { class: "w-full space-y-1.5",
                        Label { html_for: "name", { t!(tr::CON_LOGIN_USERNAME_LABEL) } }
                        Input { id: "name", placeholder: t!(tr::CON_LOGIN_USERNAME_PLACEHOLDER), }
                    }
                    div { class: "w-full space-y-1.5",
                        Label { html_for: "password", { t!(tr::CON_LOGIN_PASSWORD_LABEL) } }
                        Input { id: "password", r#type: "password", placeholder: "••••••••" }
                    }
                }
                CardFooter { class: "flex flex-col gap-3 px-5 pb-5",
                    Button {
                        class: "btn btn-primary w-full",
                        onclick: move |_| { let _ = set_locale("uk"); },
                        { t!(tr::CON_LOGIN_SIGNIN) }
                    }

                    a {
                        href: "#",
                        class: "text-xs text-center text-base-content/50 hover:text-primary transition-colors",
                        { t!(tr::CON_LOGIN_FORGOT_ACCESS_KEY) }
                    }
                }
            }

            footer { class: "text-center text-xs text-base-content/50",
                { t!(tr::CON_LOGIN_FOOTER, version: env!("CARGO_PKG_VERSION")) }
            }
        }
    }
}
