pub mod stops;
pub mod journey;
pub mod service_journey;

use crate::entur::EnturClient;
use axum::Router;

pub fn api_router(entur: EnturClient) -> Router {
    Router::new()
        .merge(stops::router())
        .merge(journey::router())
        .merge(service_journey::router())
        .with_state(entur)
}
