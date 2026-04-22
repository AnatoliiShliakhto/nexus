use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Outline,
    Ghost,
    Info,
    Success,
    Warning,
    Error,
}

impl ButtonVariant {
    #[must_use]
    pub const fn class(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Primary => "btn-primary",
            Self::Secondary => "btn-secondary",
            Self::Outline => "btn-outline",
            Self::Ghost => "btn-ghost",
            Self::Info => "btn-info",
            Self::Success => "btn-success",
            Self::Warning => "btn-warning",
            Self::Error => "btn-error",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let base = attributes!(button { class: "btn", class: variant.class() });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        button {
            onclick: move |event| {
                if let Some(f) = &onclick {
                    f.call(event);
                }
            },
            onmousedown: move |event| {
                if let Some(f) = &onmousedown {
                    f.call(event);
                }
            },
            onmouseup: move |event| {
                if let Some(f) = &onmouseup {
                    f.call(event);
                }
            },
            ..merged,
            {children}
        }
    }
}
