use super::{EnturClient, ET_CLIENT_NAME};
use kyss_shared::Stop;
use serde::Deserialize;

const GEOCODER_URL: &str = "https://api.entur.io/geocoder/v1/autocomplete";

/// Raw response feature from EnTur Geocoder (Pelias format)
#[derive(Debug, Deserialize)]
struct GeocoderResponse {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    properties: FeatureProperties,
    geometry: Geometry,
}

#[derive(Debug, Deserialize)]
struct FeatureProperties {
    id: String,
    name: String,
    label: Option<String>,
    layer: Option<String>,
    category: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Geometry {
    coordinates: Vec<f64>,
}

impl EnturClient {
    pub async fn search_stops(&self, query: &str, lat: Option<f64>, lon: Option<f64>) -> Result<Vec<Stop>, String> {
        let mut url = format!(
            "{}?text={}&lang=no&size=10&layers=venue",
            GEOCODER_URL,
            urlencoding::encode(query)
        );

        if let (Some(lat), Some(lon)) = (lat, lon) {
            url.push_str(&format!("&focus.point.lat={}&focus.point.lon={}", lat, lon));
        }

        let resp = self
            .http
            .get(&url)
            .header("ET-Client-Name", ET_CLIENT_NAME)
            .send()
            .await
            .map_err(|e| format!("Geocoder request failed: {}", e))?;

        let geocoder_resp: GeocoderResponse = resp
            .json()
            .await
            .map_err(|e| format!("Geocoder parse failed: {}", e))?;

        let stops = geocoder_resp
            .features
            .into_iter()
            .filter(|f| {
                f.properties.layer.as_deref() == Some("venue")
            })
            .map(|f| {
                let (lon, lat) = if f.geometry.coordinates.len() >= 2 {
                    (f.geometry.coordinates[0], f.geometry.coordinates[1])
                } else {
                    (0.0, 0.0)
                };

                Stop {
                    id: f.properties.id.clone(),
                    name: f.properties.label.unwrap_or(f.properties.name),
                    lat,
                    lon,
                    quay_id: None,
                    category: f.properties.category.and_then(|c| c.into_iter().next()),
                }
            })
            .collect();

        Ok(stops)
    }
}
