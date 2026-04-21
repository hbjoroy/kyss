pub mod stops;
pub mod journey;

use crate::entur::EnturClient;
use axum::Router;

pub fn api_router(entur: EnturClient) -> Router {
    Router::new()
        .merge(stops::router())
        .merge(journey::router())
        .with_state(entur)
}
