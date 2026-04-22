use axum::routing::get;
use axum::{Json, Router};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

const HOST: [u8; 4] = [0, 0, 0, 0];
const PORT: u16 = 3333;

#[derive(OpenApi)]
#[openapi(info(
    title = "Nexus • Core API Documentation",
    version = env!("CARGO_PKG_VERSION"),
    description = "## WASI Components API",
    contact(name = "Anatolii Shliakhto", email = "a.shlyakhto@gmail.com")
))]
struct RootDoc;

#[tokio::main]
async fn main() {
    println!("> Starting Local Swagger & Scalar Server...");

    let mut openapi = RootDoc::openapi();

    // openapi.merge(nx_ingress::openapi::generate());
    // openapi.merge(nx_dpop::openapi::generate());

    let app = Router::new()
        .merge(Scalar::with_url("/", openapi.clone()))
        .route("/openapi.json", get(move || async { Json(openapi) }));

    let addr = SocketAddr::from((HOST, PORT));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind to address");

    print_startup_banner(PORT);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start server");
}

fn print_startup_banner(port: u16) {
    println!("\x1b[36m");
    println!(" 🛡️  Nexus API ARCHITECT • DOCUMENTATION SERVER");
    println!(" --------------------------------------------");
    println!("\x1b[0m");
    println!(" 🚀 Status:   \x1b[32mActive\x1b[0m");
    println!(" 💎 Scalar:   \x1b[34mhttp://localhost:{port}\x1b[0m");
    println!(" 📜 OpenAPI:  \x1b[34mhttp://localhost:{port}/openapi.json\x1b[0m");
    println!("\n\x1b[90m Press Ctrl+C to terminate the process...\x1b[0m\n");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
    println!("\n\x1b[33m[SHUTDOWN]\x1b[0m Terminating Documentation Server...");
}
