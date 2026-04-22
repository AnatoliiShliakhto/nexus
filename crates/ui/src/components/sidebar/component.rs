use crate::components::button::{Button, ButtonVariant};
use crate::components::separator::Separator;
use crate::components::sheet::{
    Sheet, SheetContent, SheetDescription, SheetHeader, SheetSide, SheetTitle,
};
use crate::components::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::core::use_drop;
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::use_controlled;

// constants
const SIDEBAR_WIDTH: &str = "16rem";
const SIDEBAR_WIDTH_MOBILE: &str = "18rem";
const SIDEBAR_WIDTH_ICON: &str = "calc(3rem + 1px)";
const SIDEBAR_KEYBOARD_SHORTCUT: &str = "b";
const MOBILE_BREAKPOINT: u32 = 768;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarState {
    #[default]
    Expanded,
    Collapsed,
}

impl SidebarState {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Expanded => "expanded",
            Self::Collapsed => "collapsed",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

impl SidebarSide {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

impl SidebarVariant {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::Floating => "floating",
            Self::Inset => "inset",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

impl SidebarCollapsible {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Offcanvas => "offcanvas",
            Self::Icon => "icon",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct SidebarCtx {
    pub state: Memo<SidebarState>,
    pub side: Signal<SidebarSide>,
    pub is_mobile: Signal<bool>,
    // From use_controlled:
    open: Memo<bool>,
    set_open: Callback<bool>,
    // Mobile state:
    open_mobile: Signal<bool>,
}

impl SidebarCtx {
    /// Toggle the sidebar open/closed state
    pub fn toggle(&self) {
        if (self.is_mobile)() {
            let current = (self.open_mobile)();
            let mut open_mobile = self.open_mobile;
            open_mobile.set(!current);
        } else {
            self.set_open.call(!self.open());
        }
    }

    /// Set the mobile sidebar open state
    pub fn set_open_mobile(&self, value: bool) {
        let mut open_mobile = self.open_mobile;
        open_mobile.set(value);
    }

    /// Get the current open state (desktop)
    #[must_use]
    pub fn open(&self) -> bool {
        self.open.cloned()
    }
}

#[must_use]
pub fn use_sidebar() -> SidebarCtx {
    use_context::<SidebarCtx>()
}

#[must_use]
pub fn use_is_mobile() -> Signal<bool> {
    let mut is_mobile = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let js_code = format!(
                r"
                function checkMobile() {{
                    return window.innerWidth < {MOBILE_BREAKPOINT};
                }}
                function handleResize() {{
                    dioxus.send(checkMobile());
                }}
                window.__sidebarResizeHandler = handleResize;
                window.addEventListener('resize', window.__sidebarResizeHandler);
                dioxus.send(checkMobile());
                "
            );
            let mut eval = document::eval(&js_code);

            while let Ok(result) = eval.recv::<bool>().await {
                is_mobile.set(result);
            }
        });
    });

    use_drop(|| {
        _ = document::eval(
            r"
            window.removeEventListener('resize', window.__sidebarResizeHandler);
            delete window.__sidebarResizeHandler;
            ",
        );
    });

    is_mobile
}

#[component]
pub fn SidebarProvider(
    #[props(default = true)] default_open: bool,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let is_mobile = use_is_mobile();
    let side = use_signal(|| SidebarSide::Left);
    let open_mobile = use_signal(|| false);

    let (open, set_open) = use_controlled(open, default_open, on_open_change);

    let state =
        use_memo(move || if open() { SidebarState::Expanded } else { SidebarState::Collapsed });

    let ctx = SidebarCtx { state, side, is_mobile, open, set_open, open_mobile };

    use_context_provider(|| ctx);

    use_effect(move || {
        spawn(async move {
            let js_code = format!(
                r"
                function sidebarKeyHandler(event) {{
                    if (event.key === '{SIDEBAR_KEYBOARD_SHORTCUT}' && (event.metaKey || event.ctrlKey)) {{
                        event.preventDefault();
                        dioxus.send(true);
                    }}
                }}
                window.__sidebarKeyHandler = sidebarKeyHandler;
                window.addEventListener('keydown', window.__sidebarKeyHandler);
                ",
            );
            let mut eval = document::eval(&js_code);

            loop {
                if eval.recv::<bool>().await.is_ok() {
                    ctx.toggle();
                }
            }
        });
    });

    use_drop(|| {
        _ = document::eval(
            r"
            window.removeEventListener('keydown', window.__sidebarKeyHandler);
            delete window.__sidebarKeyHandler;
            ",
        );
    });

    let sidebar_style = format!(
        r"
        --sidebar-width: {SIDEBAR_WIDTH};
        --sidebar-width-mobile: {SIDEBAR_WIDTH_MOBILE};
        --sidebar-width-icon: {SIDEBAR_WIDTH_ICON};
        "
    );

    let base = attributes!(div {
        class: "sidebar-wrapper",
        "data-slot": "sidebar-wrapper",
        style: sidebar_style,
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn Sidebar(
    #[props(default)] side: SidebarSide,
    #[props(default)] variant: SidebarVariant,
    #[props(default)] collapsible: SidebarCollapsible,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let ctx = use_sidebar();
    let mut ctx_side = ctx.side;
    if *ctx_side.peek() != side {
        ctx_side.set(side);
    }

    let is_mobile = ctx.is_mobile;
    let state = ctx.state;
    let open_mobile = ctx.open_mobile;

    if collapsible == SidebarCollapsible::None {
        let base = attributes!(div {
            class: "sidebar sidebar-static",
            "data-slot": "sidebar",
        });
        let merged = merge_attributes(vec![base, attributes]);

        return rsx! {
            div { ..merged, {children} }
        };
    }

    if is_mobile() {
        let sheet_side = match side {
            SidebarSide::Left => SheetSide::Left,
            SidebarSide::Right => SheetSide::Right,
        };

        return rsx! {
            Sheet {
                open: open_mobile(),
                on_open_change: move |v| ctx.set_open_mobile(v),
                SheetContent {
                    side: sheet_side,
                    class: "sidebar-sheet",
                    "data-sidebar": "sidebar",
                    "data-slot": "sidebar",
                    "data-mobile": "true",
                    SheetHeader { class: "sr-only",
                        SheetTitle { "Sidebar" }
                        SheetDescription { "Displays the mobile sidebar." }
                    }
                    div { class: "sidebar-mobile-inner", {children} }
                }
            }
        };
    }

    let collapsible_str =
        if state() == SidebarState::Collapsed { collapsible.as_str() } else { "" };

    let container_base = attributes!(div {
        class: "sidebar-container",
        "data-slot": "sidebar-container",
    });
    let container_attrs = merge_attributes(vec![container_base, attributes]);

    rsx! {
        div {
            class: "sidebar-desktop",
            "data-state": state().as_str(),
            "data-collapsible": collapsible_str,
            "data-variant": variant.as_str(),
            "data-side": side.as_str(),
            "data-slot": "sidebar",
            div { class: "sidebar-gap", "data-slot": "sidebar-gap" }
            div {
                ..container_attrs,
                div {
                    class: "sidebar-inner",
                    "data-sidebar": "sidebar",
                    "data-slot": "sidebar-inner",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn SidebarTrigger(
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_sidebar();

    let base = attributes!(button {
        class: "sidebar-trigger",
        "data-sidebar": "trigger",
        "data-slot": "sidebar-trigger",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            onclick: move |e| {
                if let Some(handler) = &onclick {
                    handler.call(e);
                }
                ctx.toggle();
            },
            attributes: merged,
            i {
                class: "icon-[ph--sidebar-duotone] size-4 hover:text-primary",
                class: "transition-colors duration-200 ease-in-out",
            }
            span { class: "sr-only", "Toggle Sidebar" }
        }
    }
}

#[component]
pub fn SidebarRail(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    let ctx = use_sidebar();

    let base = attributes!(button {
        class: "sidebar-rail",
        "data-sidebar": "rail",
        "data-slot": "sidebar-rail",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        button {
            aria_label: "Toggle Sidebar",
            tabindex: -1,
            onclick: move |_| ctx.toggle(),
            title: "Toggle Sidebar",
            ..merged,
        }
    }
}

#[component]
pub fn SidebarInset(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(main {
        class: "sidebar-inset",
        "data-slot": "sidebar-inset",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        main { ..merged, {children} }
    }
}

#[component]
pub fn SidebarHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-header",
        "data-slot": "sidebar-header",
        "data-sidebar": "header",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-content",
        "data-slot": "sidebar-content",
        "data-sidebar": "content",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-footer",
        "data-slot": "sidebar-footer",
        "data-sidebar": "footer",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarSeparator(
    #[props(default = true)] horizontal: bool,
    #[props(default = true)] decorative: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-separator",
        "data-slot": "sidebar-separator",
        "data-sidebar": "separator",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        Separator { horizontal, decorative, attributes: merged }
    }
}

#[component]
pub fn SidebarGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-group",
        "data-slot": "sidebar-group",
        "data-sidebar": "group",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarGroupLabel(
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-group-label",
        "data-slot": "sidebar-group-label",
        "data-sidebar": "group-label",
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            div { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarGroupAction(
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: "sidebar-group-action",
        "data-slot": "sidebar-group-action",
        "data-sidebar": "group-action",
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            button { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarGroupContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-group-content",
        "data-slot": "sidebar-group-content",
        "data-sidebar": "group-content",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenu(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(ul {
        class: "sidebar-menu",
        "data-slot": "sidebar-menu",
        "data-sidebar": "menu",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        ul { ..merged, {children} }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SidebarMenuItemCtx {
    pub is_expanded: Signal<bool>,
}

impl SidebarMenuItemCtx {
    pub fn toggle(&mut self) {
        let current = (self.is_expanded)();
        self.is_expanded.set(!current);
    }
}

#[component]
pub fn SidebarMenuItem(
    #[props(default = false)] default_expanded: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let is_expanded = use_signal(|| default_expanded);
    use_context_provider(|| SidebarMenuItemCtx { is_expanded });

    let base = attributes!(li {
        class: "sidebar-menu-item group",
        "data-expanded": if is_expanded() { "true" } else { "false" },
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! { li { ..merged, {children} } }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SidebarMenuButtonVariant {
    #[default]
    Default,
    Outline,
}

impl SidebarMenuButtonVariant {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SidebarMenuButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl SidebarMenuButtonSize {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

#[component]
pub fn SidebarMenuButton(
    #[props(default = false)] is_active: bool,
    #[props(default)] variant: SidebarMenuButtonVariant,
    #[props(default)] size: SidebarMenuButtonSize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    #[props(default)] tooltip: Option<Element>,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let ctx = use_sidebar();
    let state = ctx.state;
    let item_ctx = try_consume_context::<SidebarMenuItemCtx>();

    let handle_click = move |e: MouseEvent| {
        if state() == SidebarState::Expanded
            && let Some(mut c) = item_ctx
        {
            c.toggle();
        }
        if let Some(handler) = &onclick {
            handler.call(e);
        }
    };

    let base_attrs = attributes!(button {
        class: "sidebar-menu-button",
        "data-slot": "sidebar-menu-button",
        "data-sidebar": "menu-button",
        "data-size": size.as_str(),
        "data-variant": variant.as_str(),
        "data-active": if is_active { "true" } else { "false" },
        onclick: handle_click,
    });

    let final_attrs = merge_attributes(vec![base_attrs, attributes]);

    if let Some(tooltip_content) = tooltip {
        let is_mobile = (ctx.is_mobile)();
        let hidden = state() != SidebarState::Collapsed || is_mobile;

        let r_as = r#as;
        let children_inner = children;
        let final_attrs_inner = final_attrs;

        rsx! {
            Tooltip {
                disabled: hidden,
                TooltipTrigger {
                    as: move |t_attrs| {
                        let merged = merge_attributes(vec![t_attrs, final_attrs_inner.clone()]);
                        if let Some(dynamic) = &r_as {
                            dynamic.call(merged)
                        } else {
                            rsx! { button { ..merged, {children_inner.clone()} } }
                        }
                    }
                }
                TooltipContent {
                    side: match (ctx.side)() {
                        SidebarSide::Left => dioxus_primitives::ContentSide::Right,
                        SidebarSide::Right => dioxus_primitives::ContentSide::Left,
                    },
                    {tooltip_content}
                }
            }
        }
    } else if let Some(dynamic) = &r#as {
        dynamic.call(final_attrs)
    } else {
        rsx! { button { ..final_attrs, {children} } }
    }
}

#[component]
pub fn SidebarMenuAction(
    #[props(default = false)] show_on_hover: bool,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: "sidebar-menu-action",
        "data-slot": "sidebar-menu-action",
        "data-sidebar": "menu-action",
        "data-show-on-hover": if show_on_hover { "true" } else { "false" },
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            button { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarMenuBadge(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-menu-badge",
        "data-slot": "sidebar-menu-badge",
        "data-sidebar": "menu-badge",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenuSkeleton(
    #[props(default = false)] show_icon: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "sidebar-menu-skeleton",
        "data-slot": "sidebar-menu-skeleton",
        "data-sidebar": "menu-skeleton",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            if show_icon {
                div { class: "skeleton sidebar-menu-skeleton-icon" }
            }
            div { class: "skeleton sidebar-menu-skeleton-text", width: "70%" }
        }
    }
}

#[component]
pub fn SidebarMenuSub(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(ul {
        class: "sidebar-menu-sub",
        "data-slot": "sidebar-menu-sub",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        ul { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenuSubItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(li {
        class: "sidebar-menu-sub-item",
        "data-slot": "sidebar-menu-sub-item",
        "data-sidebar": "menu-sub-item",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        li { ..merged, {children} }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SidebarMenuSubButtonSize {
    Sm,
    #[default]
    Md,
}

impl SidebarMenuSubButtonSize {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

#[component]
pub fn SidebarMenuSubButton(
    #[props(default = false)] is_active: bool,
    #[props(default)] size: SidebarMenuSubButtonSize,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(a {
        class: "sidebar-menu-sub-button",
        "data-active": if is_active { "true" } else { "false" },
        "data-size": size.as_str(),
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            a { ..merged, {children} }
        }
    }
}
