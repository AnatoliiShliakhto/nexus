use crate::modules::home::views::HomePage;
use crate::shared::components::layouts::{CenteredLayout, MainLayout};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Routable)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Route {
    #[layout(MainLayout)]
    #[redirect("/:.._segments", |_segments: Vec<String>| Route::HomePage {})]
    #[route("/")]
    HomePage {},
    // #[/layout]
    // #[layout(CenteredLayout)]
    // #[route("/login")]
    // LoginPage {},
    // #[route("/:..segments")]
    // PageNotFound { segments: Vec<String> },
}
