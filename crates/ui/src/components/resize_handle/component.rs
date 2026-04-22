use ::dioxus::{
    desktop::{tao::window::ResizeDirection, use_window},
    prelude::*,
};

/// A transparent, draggable handle for resizing a borderless desktop window.
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     div { class: "relative h-screen w-full",
///         // Main content here...
///         ResizeHandle {}
///     }
/// }
/// ```
#[component]
pub fn ResizeHandle() -> Element {
    let window = use_window();
    rsx! {
        div {
            class: "absolute bottom-0 right-0 w-4 h-4 cursor-se-resize",
            onmousedown: move |_evt| {
                window.drag_resize_window(ResizeDirection::SouthEast).ok();
            },
        }
    }
}
