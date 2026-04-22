use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub fn TextAreaDemo() -> Element {
    rsx! {

        Card {
            CardHeader {
                CardTitle { "Text Area", }
            }
            CardContent {
                class: "card-content flex",
                TextArea {
                    variant: TextareaVariant::Outline,
                    auto_height: true,
                }
            }
        }
    }
}
