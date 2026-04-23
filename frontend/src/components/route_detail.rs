use kyss_shared::{Leg, ServiceJourneyRealtime, TransportMode, TripPattern};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

#[component]
pub fn RouteDetailView(pattern: TripPattern) -> impl IntoView {
    let now_signal = RwSignal::new(current_time_ms());
    let legs_signal = RwSignal::new(pattern.legs);

    // Collect service journey IDs for polling
    let sj_ids: Vec<(usize, String)> = legs_signal
        .get_untracked()
        .iter()
        .enumerate()
        .filter_map(|(i, leg)| leg.service_journey_id.as_ref().map(|id| (i, id.clone())))
        .collect();

    Effect::new(move |_| {
        use gloo_timers::callback::Interval;
        let sj_ids = sj_ids.clone();

        // Initial real-time fetch
        for (leg_idx, sj_id) in sj_ids.iter().cloned() {
            let legs_signal = legs_signal;
            spawn_local(async move {
                if let Ok(rt) = fetch_service_journey_realtime(&sj_id).await {
                    legs_signal.update(|legs| {
                        if let Some(leg) = legs.get_mut(leg_idx) {
                            update_leg_from_realtime(leg, &rt);
                        }
                    });
                }
            });
        }

        // Tick every 30 seconds: update time + fetch real-time
        let interval = Interval::new(30_000, move || {
            now_signal.set(current_time_ms());
            for (leg_idx, sj_id) in sj_ids.clone() {
                let legs_signal = legs_signal;
                spawn_local(async move {
                    if let Ok(rt) = fetch_service_journey_realtime(&sj_id).await {
                        legs_signal.update(|legs| {
                            if let Some(leg) = legs.get_mut(leg_idx) {
                                update_leg_from_realtime(leg, &rt);
                            }
                        });
                    }
                });
            }
        });
        interval.forget();
    });

    view! {
        <div class="route-detail">
            {move || {
                legs_signal
                    .get()
                    .into_iter()
                    .map(|leg| {
                        view! { <LegTimeline leg=leg now=now_signal /> }
                    })
                    .collect_view()
            }}
        </div>
    }
}

/// Match real-time estimated calls back to our leg's stops by name.
fn update_leg_from_realtime(leg: &mut Leg, rt: &ServiceJourneyRealtime) {
    // Find departure stop (from_name) and arrival stop (to_name) in real-time data
    for call in &rt.estimated_calls {
        if call.name == leg.from_name {
            if let Some(ref t) = call.expected_departure {
                leg.expected_start = t.clone();
            }
        }
        if call.name == leg.to_name {
            if let Some(ref t) = call.expected_arrival {
                leg.expected_end = t.clone();
            }
        }
    }

    // Update intermediate stops
    for stop in &mut leg.intermediate_stops {
        if let Some(call) = rt.estimated_calls.iter().find(|c| c.name == stop.name) {
            stop.expected_arrival = call.expected_arrival.clone();
            stop.expected_departure = call.expected_departure.clone();
        }
    }
}

async fn fetch_service_journey_realtime(
    sj_id: &str,
) -> Result<ServiceJourneyRealtime, String> {
    let window = web_sys::window().unwrap();
    let url = format!("/api/service-journey/{}", urlencoding::encode(sj_id));

    let request = web_sys::Request::new_with_str(&url)
        .map_err(|e| format!("{:?}", e))?;

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp.dyn_into().map_err(|e| format!("{:?}", e))?;

    if !resp.ok() {
        return Err("Service journey fetch failed".to_string());
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().unwrap())
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))
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

    let from_name = leg.from_name.clone();
    let to_name = leg.to_name.clone();
    let dep_time = leg.expected_start.clone();
    let arr_time = leg.expected_end.clone();
    let aimed_dep = leg.aimed_start.clone();
    let aimed_arr = leg.aimed_end.clone();
    let intermediate = leg.intermediate_stops;
    let stop_count = intermediate.len() + 2;

    view! {
        <div class="timeline-leg">
            <div class="timeline-leg-header">
                <span class="timeline-mode-icon">{mode_icon}</span>
                <span class="timeline-line-label">{line_label}</span>
                <span class="timeline-stop-count">{stop_count}" stopp"</span>
            </div>
            <div class="timeline-stops">
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

                {intermediate
                    .into_iter()
                    .map(|stop| {
                        let time = stop
                            .expected_arrival
                            .clone()
                            .or(stop.aimed_arrival.clone())
                            .unwrap_or_default();
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

    let delayed = aimed_time.as_ref().map(|a| a != &time).unwrap_or(false);
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
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        dt.timestamp_millis() as f64
    } else if iso.find('T').is_some() {
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
