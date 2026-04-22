use crate::utils::ElementExt;
use dioxus::prelude::*;
use dioxus_primitives::switch::{self, SwitchProps};

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    rsx! {
        label {
            class: "switch-wrapper",
            "data-disabled": props.disabled,

            switch::Switch {
                class: "switch-root",
                checked: props.checked,
                default_checked: props.default_checked,
                disabled: props.disabled,
                required: props.required,
                name: props.name,
                value: props.value,
                on_checked_change: props.on_checked_change,
                attributes: props.attributes,

                switch::SwitchThumb {
                    class: "switch-thumb"
                }
            }

            if props.children.has_content() {
                span { class: "switch-text", { props.children } }
            }
        }
    }
}
