use super::{EnturClient, ET_CLIENT_NAME};
use kyss_shared::{IntermediateStop, ServiceJourneyRealtime};
use serde::{Deserialize, Serialize};

const JOURNEY_PLANNER_URL: &str = "https://api.entur.io/journey-planner/v3/graphql";

const SERVICE_JOURNEY_QUERY: &str = r#"
query ServiceJourney($id: String!) {
  serviceJourney(id: $id) {
    estimatedCalls {
      quay {
        stopPlace {
          name
        }
      }
      aimedArrivalTime
      expectedArrivalTime
      aimedDepartureTime
      expectedDepartureTime
    }
  }
}
"#;

#[derive(Serialize)]
struct GraphQLRequest {
    query: &'static str,
    variables: ServiceJourneyVars,
}

#[derive(Serialize)]
struct ServiceJourneyVars {
    id: String,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<SJData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SJData {
    service_journey: Option<RawServiceJourney>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawServiceJourney {
    estimated_calls: Vec<RawEstimatedCall>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEstimatedCall {
    quay: RawQuay,
    aimed_arrival_time: Option<String>,
    expected_arrival_time: Option<String>,
    aimed_departure_time: Option<String>,
    expected_departure_time: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawQuay {
    stop_place: RawStopPlace,
}

#[derive(Deserialize)]
struct RawStopPlace {
    name: String,
}

impl EnturClient {
    pub async fn get_service_journey_realtime(
        &self,
        service_journey_id: &str,
    ) -> Result<ServiceJourneyRealtime, String> {
        let request = GraphQLRequest {
            query: SERVICE_JOURNEY_QUERY,
            variables: ServiceJourneyVars {
                id: service_journey_id.to_string(),
            },
        };

        let resp = self
            .http
            .post(JOURNEY_PLANNER_URL)
            .header("ET-Client-Name", ET_CLIENT_NAME)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Service journey request failed: {}", e))?;

        let gql_resp: GraphQLResponse = resp
            .json()
            .await
            .map_err(|e| format!("Service journey parse failed: {}", e))?;

        if let Some(errors) = gql_resp.errors {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(format!("GraphQL errors: {}", msgs.join(", ")));
        }

        let data = gql_resp.data.ok_or("No data in service journey response")?;
        let sj = data
            .service_journey
            .ok_or("Service journey not found")?;

        let estimated_calls = sj
            .estimated_calls
            .into_iter()
            .map(|ec| IntermediateStop {
                name: ec.quay.stop_place.name,
                aimed_arrival: ec.aimed_arrival_time,
                expected_arrival: ec.expected_arrival_time,
                aimed_departure: ec.aimed_departure_time,
                expected_departure: ec.expected_departure_time,
            })
            .collect();

        Ok(ServiceJourneyRealtime { estimated_calls })
    }
}
