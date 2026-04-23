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
    /// Minimum transfer gap in minutes (sent as transferSlack in seconds to EnTur)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_transfer_minutes: Option<u32>,
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
    /// Intermediate stops with times (transit legs only)
    #[serde(default)]
    pub intermediate_stops: Vec<IntermediateStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineSummary {
    pub id: String,
    pub public_code: String,
    pub name: String,
}

/// An intermediate stop along a transit leg with arrival/departure times.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntermediateStop {
    pub name: String,
    /// ISO-8601 aimed (scheduled) arrival
    pub aimed_arrival: Option<String>,
    /// ISO-8601 expected (real-time) arrival
    pub expected_arrival: Option<String>,
    /// ISO-8601 aimed (scheduled) departure
    pub aimed_departure: Option<String>,
    /// ISO-8601 expected (real-time) departure
    pub expected_departure: Option<String>,
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
pub struct TimePeriod {
    pub id: String,
    pub name: String,
    pub icon: String,
    /// Start of period as "HH:MM"
    pub start: String,
    /// End of period as "HH:MM"
    pub end: String,
}

impl TimePeriod {
    pub fn defaults() -> Vec<TimePeriod> {
        vec![
            TimePeriod {
                id: "morning".into(),
                name: "Morgen".into(),
                icon: "🌅".into(),
                start: "06:30".into(),
                end: "09:00".into(),
            },
            TimePeriod {
                id: "mid-morning".into(),
                name: "Formiddag".into(),
                icon: "☀️".into(),
                start: "09:00".into(),
                end: "11:30".into(),
            },
            TimePeriod {
                id: "lunch".into(),
                name: "Lunsj".into(),
                icon: "🍽️".into(),
                start: "11:30".into(),
                end: "13:00".into(),
            },
            TimePeriod {
                id: "afternoon".into(),
                name: "Ettermiddag".into(),
                icon: "🌤️".into(),
                start: "13:00".into(),
                end: "15:30".into(),
            },
            TimePeriod {
                id: "end-of-work".into(),
                name: "Fra jobb".into(),
                icon: "🏠".into(),
                start: "15:00".into(),
                end: "17:30".into(),
            },
            TimePeriod {
                id: "evening".into(),
                name: "Kveld".into(),
                icon: "🌙".into(),
                start: "18:00".into(),
                end: "22:00".into(),
            },
        ]
    }
}

pub const DEFAULT_MIN_TRANSFER_MINUTES: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub version: u32,
    pub trip_types: Vec<TripType>,
    #[serde(default = "TimePeriod::defaults")]
    pub time_periods: Vec<TimePeriod>,
    #[serde(default = "default_min_transfer")]
    pub min_transfer_minutes: u32,
}

fn default_min_transfer() -> u32 {
    DEFAULT_MIN_TRANSFER_MINUTES
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: 1,
            trip_types: Vec::new(),
            time_periods: TimePeriod::defaults(),
            min_transfer_minutes: DEFAULT_MIN_TRANSFER_MINUTES,
        }
    }
}
