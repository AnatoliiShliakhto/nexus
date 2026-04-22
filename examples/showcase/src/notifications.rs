use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn NotificationsDemo() -> Element {
    let notify = use_notifications();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Notification center", }
            }
            CardContent {
                div { class: "flex flex-col gap-3",
                    h3 { class: "text-xs uppercase tracking-wider text-base-content/50", "Toasted Notifications" }
                    div { class: "flex flex-wrap gap-4",
                        button {
                            class: "btn btn-success",
                            onclick: move |_| notify
                            .new("Project Updated")
                            .desc("All changes have been synchronized with the remote server.")
                            .success()
                            .send(),
                            "Push Success"
                        }
                        button {
                            class: "btn btn-error",
                            onclick: move |_| notify
                            .new("Connection Failed")
                            .desc("Unable to reach the backend service. Retrying in 5s...")
                            .error()
                            .send(),
                            "Push Error"
                        }
                    }
                }

                Separator { horizontal: false }

                div { class: "flex flex-col gap-3",
                    h3 { class: "text-xs uppercase tracking-wider text-base-content/50", "Silent Logs (In-App Only)" }
                    div { class: "flex flex-wrap gap-4",
                        button {
                            class: "btn btn-info",
                            onclick: move |_| notify
                            .new("System Update")
                            .desc("New component version (v2.1.0) was loaded in background.")
                            .silent()
                            .send(),
                            "Log Info"
                        }
                        button {
                            class: "btn btn-warning",
                            onclick: move |_| notify
                            .new("Memory Usage")
                            .desc("Application is consuming more than 500MB of RAM.")
                            .warning()
                            .silent()
                            .send(),
                            "Log Warning"
                        }
                    }
                }
            }
        }
    }
}
