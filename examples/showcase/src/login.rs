use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn LoginFormDemo() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Login form" }
                CardDescription { "The login form example" }
                CardAction {
                    Button {
                        variant: ButtonVariant::Primary,
                        "Push me!"
                    }
                }
            }
            CardContent {
                class: "card-content gap-3",
                div {
                    class: "flex w-full flex-col gap-.5",
                    Label {
                        html_for: "name",
                        i { class: "icon-[ph--person] text-info" }
                        "Name"
                    }

                    Input {
                        id: "name",
                        class: "input w-full",
                        placeholder: "Enter your name",
                    }
                }
                div {
                    class: "flex w-full flex-col gap-.5",
                    Label {
                        html_for: "password",
                        i { class: "icon-[ph--key-duotone] text-info" }
                        "Password"
                    }
                    Input {
                        id: "password",
                        r#type: "password",
                        class: "input w-full",
                        placeholder: "Enter your password"
                    }
                }
            }
            CardFooter {
                Button { "Submit" }
            }
        }
    }
}
