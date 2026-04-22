use crate::dpop;
use utoipa::OpenApi;

pub(crate) const TAG: &str = "⚙️ nx-dpop-component";

#[derive(Debug, OpenApi)]
#[openapi(
    tags((name = TAG, description = env!("CARGO_PKG_DESCRIPTION"))),
    paths(
        dpop::verify_proof_handler,
        dpop::compute_thumbprint_handler,
    ),
)]
struct ApiDoc;

pub fn generate() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
