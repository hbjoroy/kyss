use kyss_shared::{Leg, TransportMode, TripPattern};
use leptos::prelude::*;

#[component]
pub fn RouteDetailView(pattern: TripPattern) -> impl IntoView {
    let now_signal = RwSignal::new(current_time_ms());

    // Tick every 30 seconds to update live progress
    Effect::new(move |_| {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(30_000, move || {
            now_signal.set(current_time_ms());
        });
        interval.forget();
    });

    let legs = pattern.legs;

    view! {
        <div class="route-detail">
            {legs
                .into_iter()
                .map(|leg| {
                    view! { <LegTimeline leg=leg now=now_signal /> }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn LegTimeline(leg: Leg, now: RwSignal<f64>) -> impl IntoView {
    let is_walking = leg.mode == TransportMode::Foot;

    if is_walking {
        let to_name = leg.to_name.clone();
        let start_time = format_time(&leg.expected_start);
        let end_time = format_time(&leg.expected_end);
        return view! {
            <div class="timeline-walk">
                <div class="timeline-walk-icon">"🚶"</div>
                <div class="timeline-walk-info">
                    <span class="timeline-walk-label">"Gå til "{to_name}</span>
                    <span class="timeline-walk-time">{start_time}" → "{end_time}</span>
                </div>
            </div>
        }
        .into_any();
    }

    let line_label = leg
        .line
        .as_ref()
        .map(|l| {
            format!(
                "{} {}",
                l.public_code,
                leg.destination.as_deref().unwrap_or(&l.name)
            )
        })
        .unwrap_or_else(|| leg.mode.to_string());

    let mode_icon = leg.mode.to_string();

    // Build the full stop list: origin + intermediate + destination
    let from_name = leg.from_name.clone();
    let to_name = leg.to_name.clone();
    let dep_time = leg.expected_start.clone();
    let arr_time = leg.expected_end.clone();
    let aimed_dep = leg.aimed_start.clone();
    let aimed_arr = leg.aimed_end.clone();
    let intermediate = leg.intermediate_stops;
    let stop_count = intermediate.len() + 2; // origin + intermediate + destination

    view! {
        <div class="timeline-leg">
            <div class="timeline-leg-header">
                <span class="timeline-mode-icon">{mode_icon}</span>
                <span class="timeline-line-label">{line_label}</span>
                <span class="timeline-stop-count">{stop_count}" stopp"</span>
            </div>
            <div class="timeline-stops">
                // Origin stop
                {
                    let dep = dep_time.clone();
                    let aimed = aimed_dep.clone();
                    let name = from_name.clone();
                    view! {
                        <TimelineStop
                            name=name
                            time=dep.clone()
                            aimed_time=aimed
                            is_origin=true
                            is_destination=false
                            now=now
                        />
                    }
                }

                // Intermediate stops
                {intermediate
                    .into_iter()
                    .map(|stop| {
                        let time = stop.expected_arrival.clone().or(stop.aimed_arrival.clone()).unwrap_or_default();
                        let aimed = stop.aimed_arrival.clone();
                        let name = stop.name.clone();
                        view! {
                            <TimelineStop
                                name=name
                                time=time
                                aimed_time=aimed
                                is_origin=false
                                is_destination=false
                                now=now
                            />
                        }
                    })
                    .collect_view()}

                // Destination stop
                {
                    let arr = arr_time.clone();
                    let aimed = aimed_arr.clone();
                    let name = to_name.clone();
                    view! {
                        <TimelineStop
                            name=name
                            time=arr.clone()
                            aimed_time=aimed
                            is_origin=false
                            is_destination=true
                            now=now
                        />
                    }
                }
            </div>
        </div>
    }
    .into_any()
}

#[component]
fn TimelineStop(
    name: String,
    time: String,
    aimed_time: Option<String>,
    is_origin: bool,
    is_destination: bool,
    now: RwSignal<f64>,
) -> impl IntoView {
    let time_ms = parse_iso_ms(&time);
    let time_display = format_time(&time);

    let delayed = aimed_time
        .as_ref()
        .map(|a| a != &time)
        .unwrap_or(false);
    let aimed_display = if delayed {
        aimed_time.as_ref().map(|t| format_time(t))
    } else {
        None
    };

    let passed = Memo::new(move |_| {
        if time_ms > 0.0 {
            now.get() > time_ms
        } else {
            false
        }
    });

    let stop_class = move || {
        let mut cls = String::from("timeline-stop");
        if is_origin {
            cls.push_str(" timeline-stop--origin");
        }
        if is_destination {
            cls.push_str(" timeline-stop--destination");
        }
        if passed.get() {
            cls.push_str(" timeline-stop--passed");
        }
        cls
    };

    view! {
        <div class=stop_class>
            <div class="timeline-dot-col">
                <div class="timeline-dot"></div>
            </div>
            <div class="timeline-stop-time">
                <span class:delayed=delayed>{time_display.clone()}</span>
                {aimed_display.map(|a| view! { <span class="aimed-time">" "{a}</span> })}
            </div>
            <div class="timeline-stop-name">{name.clone()}</div>
        </div>
    }
}

fn format_time(iso: &str) -> String {
    if let Some(t_idx) = iso.find('T') {
        let time_part = &iso[t_idx + 1..];
        if time_part.len() >= 5 {
            return time_part[..5].to_string();
        }
    }
    iso.to_string()
}

fn parse_iso_ms(iso: &str) -> f64 {
    // Parse ISO-8601 to epoch millis for comparison with JS Date.now()
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        dt.timestamp_millis() as f64
    } else if let Some(_t_idx) = iso.find('T') {
        // Try parsing with offset like +02:00
        let s = iso.trim();
        chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%:z")
            .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f%:z"))
            .map(|dt| dt.timestamp_millis() as f64)
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

fn current_time_ms() -> f64 {
    js_sys::Date::now()
}
