use super::{EnturClient, ET_CLIENT_NAME};
use kyss_shared::{IntermediateStop, JourneyResult, Leg, LineSummary, TransportMode, TripPattern};
use serde::{Deserialize, Serialize};

const JOURNEY_PLANNER_URL: &str = "https://api.entur.io/journey-planner/v3/graphql";

/// GraphQL query for trip planning
const TRIP_QUERY: &str = r#"
query Trip($from: Location!, $to: Location!, $dateTime: DateTime, $numTripPatterns: Int, $transferSlack: Int) {
  trip(
    from: $from
    to: $to
    dateTime: $dateTime
    numTripPatterns: $numTripPatterns
    transferSlack: $transferSlack
  ) {
    tripPatterns {
      startTime
      endTime
      duration
      legs {
        mode
        fromPlace {
          name
          latitude
          longitude
        }
        toPlace {
          name
          latitude
          longitude
        }
        expectedStartTime
        expectedEndTime
        aimedStartTime
        aimedEndTime
        line {
          id
          publicCode
          name
        }
        fromEstimatedCall {
          destinationDisplay {
            frontText
          }
        }
        intermediateEstimatedCalls {
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
  }
}
"#;

#[derive(Serialize)]
struct GraphQLRequest {
    query: &'static str,
    variables: TripVariables,
}

#[derive(Serialize)]
struct TripVariables {
    from: LocationInput,
    to: LocationInput,
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    #[serde(rename = "numTripPatterns", skip_serializing_if = "Option::is_none")]
    num_trip_patterns: Option<u32>,
    #[serde(rename = "transferSlack", skip_serializing_if = "Option::is_none")]
    transfer_slack: Option<u32>,
}

#[derive(Serialize)]
struct LocationInput {
    place: String,
}

// --- GraphQL response types (internal to BFF) ---

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<TripData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct TripData {
    trip: TripResult,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TripResult {
    trip_patterns: Vec<RawTripPattern>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTripPattern {
    start_time: String,
    end_time: String,
    duration: i64,
    legs: Vec<RawLeg>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLeg {
    mode: String,
    from_place: RawPlace,
    to_place: RawPlace,
    expected_start_time: String,
    expected_end_time: String,
    aimed_start_time: Option<String>,
    aimed_end_time: Option<String>,
    line: Option<RawLine>,
    from_estimated_call: Option<RawEstimatedCall>,
    intermediate_estimated_calls: Option<Vec<RawIntermediateCall>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEstimatedCall {
    destination_display: Option<RawDestination>,
}

#[derive(Deserialize)]
struct RawPlace {
    name: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLine {
    id: String,
    public_code: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDestination {
    front_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIntermediateCall {
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

fn parse_mode(s: &str) -> TransportMode {
    match s.to_lowercase().as_str() {
        "bus" => TransportMode::Bus,
        "tram" => TransportMode::Tram,
        "rail" => TransportMode::Rail,
        "metro" => TransportMode::Metro,
        "water" => TransportMode::Water,
        "air" => TransportMode::Air,
        "coach" => TransportMode::Coach,
        "foot" => TransportMode::Foot,
        _ => TransportMode::Unknown,
    }
}

impl EnturClient {
    pub async fn plan_journey(
        &self,
        from_id: &str,
        to_id: &str,
        date_time: Option<String>,
        num_results: Option<u32>,
        min_transfer_minutes: Option<u32>,
    ) -> Result<JourneyResult, String> {
        let request = GraphQLRequest {
            query: TRIP_QUERY,
            variables: TripVariables {
                from: LocationInput { place: from_id.to_string() },
                to: LocationInput { place: to_id.to_string() },
                date_time,
                num_trip_patterns: num_results.or(Some(5)),
                transfer_slack: min_transfer_minutes.map(|m| m * 60),
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
            .map_err(|e| format!("Journey planner request failed: {}", e))?;

        let gql_resp: GraphQLResponse = resp
            .json()
            .await
            .map_err(|e| format!("Journey planner parse failed: {}", e))?;

        if let Some(errors) = gql_resp.errors {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(format!("GraphQL errors: {}", msgs.join(", ")));
        }

        let data = gql_resp.data.ok_or("No data in journey planner response")?;

        let trip_patterns = data
            .trip
            .trip_patterns
            .into_iter()
            .map(|tp| TripPattern {
                start_time: tp.start_time,
                end_time: tp.end_time,
                duration: tp.duration,
                legs: tp
                    .legs
                    .into_iter()
                    .map(|leg| Leg {
                        mode: parse_mode(&leg.mode),
                        from_name: leg.from_place.name,
                        to_name: leg.to_place.name,
                        from_lat: leg.from_place.latitude,
                        from_lon: leg.from_place.longitude,
                        to_lat: leg.to_place.latitude,
                        to_lon: leg.to_place.longitude,
                        expected_start: leg.expected_start_time,
                        expected_end: leg.expected_end_time,
                        aimed_start: leg.aimed_start_time,
                        aimed_end: leg.aimed_end_time,
                        line: leg.line.map(|l| LineSummary {
                            id: l.id,
                            public_code: l.public_code,
                            name: l.name,
                        }),
                        destination: leg.from_estimated_call
                            .and_then(|ec| ec.destination_display)
                            .map(|d| d.front_text),
                        intermediate_stops: leg
                            .intermediate_estimated_calls
                            .unwrap_or_default()
                            .into_iter()
                            .map(|ic| IntermediateStop {
                                name: ic.quay.stop_place.name,
                                aimed_arrival: ic.aimed_arrival_time,
                                expected_arrival: ic.expected_arrival_time,
                                aimed_departure: ic.aimed_departure_time,
                                expected_departure: ic.expected_departure_time,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();

        Ok(JourneyResult { trip_patterns })
    }
}
