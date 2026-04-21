use axum::{
    Router,
    extract::State,
    routing::post,
    Json,
};
use crate::entur::EnturClient;
use kyss_shared::{JourneyRequest, JourneyResult};

async fn plan_journey(
    State(entur): State<EnturClient>,
    Json(req): Json<JourneyRequest>,
) -> Result<Json<JourneyResult>, String> {
    let result = entur
        .plan_journey(&req.from, &req.to, req.date_time, req.num_results)
        .await?;
    Ok(Json(result))
}

pub fn router() -> Router<EnturClient> {
    Router::new().route("/journey", post(plan_journey))
}
