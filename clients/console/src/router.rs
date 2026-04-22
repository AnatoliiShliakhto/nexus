use crate::modules::auth::views::LoginPage;
use crate::shared::components::layouts::CenteredLayout;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Routable)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Route {
    #[layout(CenteredLayout)]
    #[redirect("/:.._segments", |_segments: Vec<String>| Route::LoginPage {})]
    #[route("/login")]
    LoginPage {},
    // #[route("/:..segments")]
    // PageNotFound { segments: Vec<String> },
}
