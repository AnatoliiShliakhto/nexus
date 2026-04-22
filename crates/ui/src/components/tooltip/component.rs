use dioxus::prelude::*;
use dioxus_primitives::tooltip::{self, TooltipContentProps, TooltipProps, TooltipTriggerProps};
use std::sync::atomic::{AtomicUsize, Ordering};

static TOOLTIP_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Default, PartialEq)]
struct TooltipContext {
    anchor_name: String,
}

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let context = use_hook(|| {
        let id = TOOLTIP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let anchor_name = format!("--tooltip-anchor-{id}");

        TooltipContext { anchor_name }
    });
    use_context_provider(|| context);

    rsx! {
        tooltip::Tooltip {
            class: "tooltip",
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let ctx = use_context::<TooltipContext>();

    rsx! {
        tooltip::TooltipTrigger {
            style: "anchor-name: {ctx.anchor_name};",
            class: "tooltip-trigger",
            id: props.id,
            r#as: props.r#as,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let ctx = use_context::<TooltipContext>();

    rsx! {
        tooltip::TooltipContent {
            style: "position-anchor: {ctx.anchor_name};",
            class: "tooltip-content",
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: props.attributes,
            {props.children}
        }
    }
}
