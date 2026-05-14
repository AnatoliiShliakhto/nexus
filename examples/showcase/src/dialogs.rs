use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn DialogDemo() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut show_alert = use_signal(|| false);

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Dialogs", }
            }
            CardContent {
                Button {
                    onclick: move |_| dialog_open.set(true),
                    "Dialog"
                }
                Button {
                    onclick: move |_| show_alert.set(true),
                    "Alert Dialog"
                }
            }
        }

        DialogRoot {
            open: dialog_open(),
            on_open_change: move |v| dialog_open.set(v),
            DialogContent {
                button {
                    class: "dialog-close",
                    r#type: "button",
                    aria_label: "Close",
                    onclick: move |_| dialog_open.set(false),
                    i { class: "icon-[ph--x-bold] size-4!" }
                }
                DialogTitle {
                    "Information"
                }
                DialogDescription {
                    "Here is some additional information about the item."
                }
            }
        }

        AlertDialogRoot {
            open: show_alert(),
            AlertDialogContent {
                AlertDialogTitle { "What can I do?" }
                AlertDialogDescription {
                    "Here is some additional information about the item."
                }
                AlertDialogActions {
                    AlertDialogCancel {
                        class: "btn btn-ghost",
                        on_click: move |_| show_alert.set(false),
                        "Cancel"
                    }
                    AlertDialogAction {
                        class: "btn btn-primary",
                        on_click: move |_| show_alert.set(false),
                        "Ok"
                    }
                }
            }
        }
    }
}
