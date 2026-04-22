use utoipa::OpenApi;

pub(crate) const TAG: &str = "⚙️ nx-access";

#[derive(Debug, OpenApi)]
#[openapi(
    tags((name = TAG, description = env!("CARGO_PKG_DESCRIPTION"))),
    paths(),
)]
struct ApiDoc;

pub fn generate() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
