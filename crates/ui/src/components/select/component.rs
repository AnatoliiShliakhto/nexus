use dioxus::prelude::*;
use dioxus_primitives::select::{
    self, SelectGroupLabelProps, SelectGroupProps, SelectItemIndicator, SelectListProps,
    SelectOptionProps, SelectProps, SelectTriggerProps, SelectValueProps,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static SELECT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Default, PartialEq)]
struct SelectContext {
    anchor_name: String,
}

#[component]
pub fn Select<T: Clone + PartialEq + 'static>(props: SelectProps<T>) -> Element {
    let context = use_hook(|| {
        let id = SELECT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let anchor_name = format!("--select-anchor-{id}");

        SelectContext { anchor_name }
    });
    use_context_provider(|| context);

    rsx! {
        select::Select {
            class: "select-root",
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            name: props.name,
            placeholder: props.placeholder,
            roving_loop: props.roving_loop,
            typeahead_timeout: props.typeahead_timeout,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let ctx = use_context::<SelectContext>();

    rsx! {
        select::SelectTrigger {
            class: "select-trigger",
            style: "anchor-name: {ctx.anchor_name};",
            attributes: props.attributes,

            {props.children}
            i { class: "select-chevron icon-[ph--caret-down-bold]" }
        }
    }
}

#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    rsx! {
        span { class: "select-value-container",
            select::SelectValue { attributes: props.attributes }
        }
    }
}

#[component]
pub fn SelectList(props: SelectListProps) -> Element {
    let ctx = use_context::<SelectContext>();

    rsx! {
        select::SelectList {
            class: "select-list",
            id: props.id,
            style: "position-anchor: {ctx.anchor_name};",
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectGroup(props: SelectGroupProps) -> Element {
    rsx! {
        select::SelectGroup {
            class: "select-group",
            disabled: props.disabled,
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectGroupLabel(props: SelectGroupLabelProps) -> Element {
    rsx! {
        select::SelectGroupLabel {
            class: "select-group-label",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectOption<T: Clone + PartialEq + 'static>(props: SelectOptionProps<T>) -> Element {
    rsx! {
        select::SelectOption::<T> {
            class: "select-option",
            value: props.value,
            text_value: props.text_value,
            disabled: props.disabled,
            id: props.id,
            index: props.index,
            attributes: props.attributes,

            span { class: "flex-1 truncate", {props.children} }

            SelectItemIndicator {
                div { class: "select-indicator ml-2",
                    i { class: "icon-[ph--check-bold]" }
                }
            }
        }
    }
}
