mod entur;
mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let entur_client = entur::EnturClient::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = routes::api_router(entur_client);

    // Serve frontend static files from dist/ with SPA fallback
    let frontend = ServeDir::new("../frontend/dist")
        .not_found_service(ServeFile::new("../frontend/dist/index.html"));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(frontend)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("BFF listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
