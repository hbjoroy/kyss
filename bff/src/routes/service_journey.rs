use axum::{
    Router,
    extract::{Path, State},
    routing::get,
    Json,
};
use crate::entur::EnturClient;
use kyss_shared::ServiceJourneyRealtime;

async fn get_realtime(
    State(entur): State<EnturClient>,
    Path(id): Path<String>,
) -> Result<Json<ServiceJourneyRealtime>, String> {
    let result = entur.get_service_journey_realtime(&id).await?;
    Ok(Json(result))
}

pub fn router() -> Router<EnturClient> {
    Router::new().route("/service-journey/{id}", get(get_realtime))
}
