use crate::utils::ElementExt;
use dioxus::prelude::*;
pub use dioxus_primitives::checkbox::CheckboxState;
use dioxus_primitives::checkbox::{self, CheckboxProps};

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    rsx! {
        label {
            class: "checkbox-wrapper",
            "data-disabled": props.disabled,

            checkbox::Checkbox {
                class: "checkbox",
                checked: props.checked,
                default_checked: props.default_checked,
                required: props.required,
                disabled: props.disabled,
                on_checked_change: props.on_checked_change,

                div { class: "checkbox-box",
                    checkbox::CheckboxIndicator {
                        class: "checkbox-indicator",
                        i { class: "icon-check icon-[ph--check-bold]" }
                        i { class: "icon-indet icon-[ph--minus-bold]" }
                    }
                }
            }

            if props.children.has_content() {
                span { class: "checkbox-text", { props.children } }
            }
        }
    }
}
