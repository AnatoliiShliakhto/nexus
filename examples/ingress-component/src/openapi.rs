use utoipa::OpenApi;

pub(crate) const TAG: &str = "🌐 nx-ingress";

#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    tags((name = TAG, description = env!("CARGO_PKG_DESCRIPTION"))),
    paths(health),
)]
struct ApiDoc;

pub fn generate() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/health",
    tag = TAG,
    operation_id = "ingress_health",
    summary = "Health Check",
    description = "Health check endpoint.",
    responses((status = 200, description = "Returns a 200 OK status.")),
)]
const fn health() {}
