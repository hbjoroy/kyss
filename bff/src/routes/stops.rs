use axum::{
    Router,
    extract::{Query, State},
    routing::get,
    Json,
};
use crate::entur::EnturClient;
use kyss_shared::Stop;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    lat: Option<f64>,
    lon: Option<f64>,
}

async fn search_stops(
    State(entur): State<EnturClient>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Stop>>, String> {
    let stops = entur.search_stops(&params.q, params.lat, params.lon).await?;
    Ok(Json(stops))
}

pub fn router() -> Router<EnturClient> {
    Router::new().route("/stops/search", get(search_stops))
}
