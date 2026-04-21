mod entur;
mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let entur_client = entur::EnturClient::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = routes::api_router(entur_client);

    // In production, serve frontend static files from dist/
    let app = Router::new()
        .nest("/api", api_routes)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("BFF listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
