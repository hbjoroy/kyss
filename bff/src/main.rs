mod entur;
mod routes;

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("LISTEN_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let dist_dir = std::env::var("FRONTEND_DIST_DIR")
        .unwrap_or_else(|_| "../frontend/dist".to_string());

    let entur_client = entur::EnturClient::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = routes::api_router(entur_client);

    let index_path = format!("{}/index.html", &dist_dir);
    let frontend = ServeDir::new(&dist_dir)
        .not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .nest("/api", api_routes)
        .fallback_service(frontend)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("BFF listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
