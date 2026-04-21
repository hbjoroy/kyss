use serde::{Deserialize, Serialize};

// --- Stop ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stop {
    /// NSR StopPlace ID, e.g. "NSR:StopPlace:59872"
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    /// Optional quay ID for direction-specific selection
    pub quay_id: Option<String>,
    /// e.g. "onstreetBus", "railStation"
    pub category: Option<String>,
}

// --- Trip Type (user-saved search) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripType {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub from_stop: Stop,
    pub to_stop: Stop,
    pub line_preferences: Vec<LinePreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinePreference {
    pub line_id: String,
    pub line_name: String,
    pub line_code: String,
    pub preferred: bool,
}

// --- Journey request/response (app-level DTOs) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyRequest {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyResult {
    pub trip_patterns: Vec<TripPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripPattern {
    /// ISO-8601 with timezone
    pub start_time: String,
    pub end_time: String,
    /// Duration in seconds
    pub duration: i64,
    pub legs: Vec<Leg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub mode: TransportMode,
    pub from_name: String,
    pub to_name: String,
    pub from_lat: f64,
    pub from_lon: f64,
    pub to_lat: f64,
    pub to_lon: f64,
    /// ISO-8601 expected departure
    pub expected_start: String,
    /// ISO-8601 expected arrival
    pub expected_end: String,
    /// Aimed (scheduled) departure
    pub aimed_start: Option<String>,
    /// Aimed (scheduled) arrival
    pub aimed_end: Option<String>,
    pub line: Option<LineSummary>,
    /// Human-readable destination (e.g. "Ljabru")
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineSummary {
    pub id: String,
    pub public_code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransportMode {
    Bus,
    Tram,
    Rail,
    Metro,
    Water,
    Air,
    Coach,
    Foot,
    Unknown,
}

impl std::fmt::Display for TransportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bus => write!(f, "🚌"),
            Self::Tram => write!(f, "🚊"),
            Self::Rail => write!(f, "🚆"),
            Self::Metro => write!(f, "🚇"),
            Self::Water => write!(f, "⛴️"),
            Self::Air => write!(f, "✈️"),
            Self::Coach => write!(f, "🚍"),
            Self::Foot => write!(f, "🚶"),
            Self::Unknown => write!(f, "🚏"),
        }
    }
}

// --- Local storage wrapper ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub version: u32,
    pub trip_types: Vec<TripType>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: 1,
            trip_types: Vec::new(),
        }
    }
}
