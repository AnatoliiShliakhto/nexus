use crate::utils::ElementExt;
use dioxus::prelude::*;
use dioxus_primitives::radio_group::{self, RadioGroupProps, RadioItemProps};

#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    rsx! {
        radio_group::RadioGroup {
            class: "radio-group",
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            horizontal: props.horizontal,
            roving_loop: props.roving_loop,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn RadioItem(props: RadioItemProps) -> Element {
    rsx! {
        label {
            class: "radio-wrapper",
            "data-disabled": props.disabled,

            radio_group::RadioItem {
                class: "radio-item",
                value: props.value,
                index: props.index,
                disabled: props.disabled,
                attributes: props.attributes,

                div { class: "radio-box",
                    div { class: "radio-dot" }
                }
            }

            if props.children.has_content() {
                span { class: "radio-text", { props.children } }
            }
        }
    }
}
