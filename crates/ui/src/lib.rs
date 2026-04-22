use dioxus::prelude::*;

pub mod components;
pub mod hooks;
pub mod macros;
pub mod utils;
pub mod widgets;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::hooks::*;
    pub use crate::utils::*;
    pub use crate::widgets::*;
    pub use crate::{enclose, t};
    pub use dioxus_primitives::*;
    pub use nx_i18n::tr;
}

#[component]
pub fn Resources() -> Element {
    #[allow(clippy::volatile_composites)]
    {
        _ = asset!("assets/fonts", AssetOptions::folder()).resolve();
    }

    rsx! {}
}
