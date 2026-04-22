use dioxus::prelude::*;
use dioxus_primitives::separator::{self, SeparatorProps};

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    rsx! {
        separator::Separator {
            class: "separator",
            horizontal: props.horizontal,
            decorative: props.decorative,
            attributes: props.attributes,
            {props.children}
        }
    }
}
