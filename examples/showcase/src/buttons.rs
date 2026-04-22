use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn ButtonsDemo() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Buttons", }
            }
            CardContent {
                Button { "Default" }
                Button { variant: ButtonVariant::Primary, "Primary" }
                Button { variant: ButtonVariant::Secondary, "Secondary" }
                Button { variant: ButtonVariant::Outline, "Outline" }
                Button { variant: ButtonVariant::Ghost, "Ghost" }
                Button { variant: ButtonVariant::Info, "Info" }
                Button { variant: ButtonVariant::Warning, "Warning" }
                Button { variant: ButtonVariant::Success, "Success" }
                        Button { variant: ButtonVariant::Error, "Error" }
                Tooltip {
                    TooltipTrigger {
                        button {
                            class: "btn btn-xs",
                            "I have tooltip!"
                        }
                    }
                    TooltipContent {
                        side: ContentSide::Left,
                        h4 { class: "mb-3 text-info text-sm;", "Tooltip" }
                        p { class: "m-0 text-xs", "This tooltip contains rich HTML content with styling." }
                    }
                }
            }
        }
    }
}
