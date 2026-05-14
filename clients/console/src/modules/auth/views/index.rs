use dioxus::prelude::*;
use heck::ToKebabCase;
use nx_error::{ErrorMetadata, ErrorMetadataExt};
use nx_ui::gather;
use nx_ui::macros::i18n::set_locale;
use nx_ui::prelude::{tr::console as tr, *};
use nx_web_client::use_web_client;
use std::ops::Add;

#[component]
pub(crate) fn LoginPage() -> Element {
    let client = use_web_client();
    let notify = use_notifications();

    let mut busy = use_signal(|| false);
    let mut err_msg = use_signal(|| Option::<String>::None);
    let mut username_len = use_signal(|| 0);

    let submit = move |evt: Event<FormData>| {
        busy.set(true);

        let (Some(username), password) = gather!(evt, username, password) else {
            err_msg.set(Some(tr::CON_INVALID_CREDENTIALS.to_owned()));
            return;
        };

        spawn(enclose!(client, notify => async move {
            if let Err(e) = client.login_with_credentials(username, password.unwrap_or_default()).await {
                let msg = e.code().to_kebab_case();
                err_msg.set(Some(msg.clone()));
                notify.new(t!(tr::CON_ERROR_TITLE)).desc(t!(msg)).error().silent().send();
            }
            busy.set(false);
        }));
    };

    rsx! {
        main {
            class: "flex flex-1 items-center justify-center min-h-0 min-w-0 overflow-hidden",
            div { class: "flex flex-col gap-6",

                div { class: "text-center space-y-2",
                    i { class: "icon-[ph--shield-check-duotone] text-5xl text-primary" }
                    h1 { class: "text-2xl font-bold tracking-tight", { t!(tr::CON_LOGIN_TITLE) } }
                    p { class: "text-base-content/60 text-sm", { t!(tr::CON_LOGIN_DESCRIPTION) } }
                }

                form {
                    id: "login-form",
                    autocomplete: "off",
                    onsubmit: submit,
                    Card { class: "card shadow-lg w-[400px] border-t-4 border-t-primary",
                        CardContent { class: "card-content gap-4 pt-6",
                            div { class: "w-full space-y-1.5",
                                Label { html_for: "username", { t!(tr::CON_LOGIN_USERNAME_LABEL) } }
                                Input {
                                    name: "username",
                                    placeholder: t!(tr::CON_LOGIN_USERNAME_PLACEHOLDER),
                                    oninput: move |evt: FormEvent| username_len.set(evt.value().len()),
                                }
                            }
                            div { class: "w-full space-y-1.5",
                                Label { html_for: "password", { t!(tr::CON_LOGIN_PASSWORD_LABEL) } }
                                Input { name: "password", r#type: "password", placeholder: "••••••••" }
                            }
                        }
                        CardFooter { class: "flex flex-col gap-3 px-5 pb-5",
                            Button {
                                class: "btn btn-primary w-full",
                                form: "login-form",
                                disabled: busy() || username_len() < 4, // todo: more complex check
                                { t!(tr::CON_LOGIN_SIGNIN) }
                            }

                            a {
                                href: "#",
                                class: "text-xs text-center text-base-content/50 hover:text-primary transition-colors",
                                { t!(tr::CON_LOGIN_FORGOT_ACCESS_KEY) }
                            }
                        }
                    }
                }

                footer { class: "text-center text-xs text-base-content/50",
                    { t!(tr::CON_LOGIN_FOOTER, version: env!("CARGO_PKG_VERSION")) }
                }
            }
        }

        DialogRoot {
            open: err_msg.read().is_some(),
            on_open_change: move |v: bool| if !v { err_msg.set(None) },
            DialogContent {
                button {
                    class: "dialog-close",
                    r#type: "button",
                    onclick: move |_| err_msg.set(None),
                    i { class: "icon-[ph--x-bold] size-4!" }
                }
                DialogTitle {
                    div { class: "text-error", { t!(tr::CON_ERROR_TITLE) } }
                }
                if let Some(msg) = &*err_msg.read() {
                    Separator {}
                    DialogDescription {
                        { t!(msg) }
                    }
                }
            }
        }
    }
}
