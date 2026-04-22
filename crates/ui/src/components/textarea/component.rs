use dioxus::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextareaVariant {
    #[default]
    Default,
    Fade,
    Outline,
    Ghost,
}

impl TextareaVariant {
    #[must_use]
    pub const fn class(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Fade => "fade",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }
}

#[component]
pub fn TextArea(
    #[props(default)] variant: TextareaVariant,
    #[props(default = false)] auto_height: bool,
    #[props(extends = textarea)] attributes: Vec<Attribute>,
    oninput: Option<EventHandler<FormEvent>>,
    children: Element,
) -> Element {
    rsx! {
        textarea {
            class: "textarea",
            class: if auto_height { "textarea-auto-height" },
            "data-variant": variant.class(),

            oninput: move |e| {
                if let Some(handler) = &oninput {
                    handler.call(e);
                }
            },

            ..attributes,
            {children}
        }
    }
}
