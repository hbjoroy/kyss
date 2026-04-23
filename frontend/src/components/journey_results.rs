use crate::components::route_detail::RouteDetailView;
use crate::components::trip_detail_view::FullTripViewSignal;
use kyss_shared::{JourneyResult, Leg, TripPattern};
use leptos::prelude::*;

#[component]
pub fn JourneyResults(result: JourneyResult) -> impl IntoView {
    if result.trip_patterns.is_empty() {
        return view! {
            <div class="no-results">
                <p>"Ingen reiser funnet"</p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="journey-results">
            {result
                .trip_patterns
                .into_iter()
                .map(|tp| {
                    view! { <TripPatternCard pattern=tp /> }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[component]
fn TripPatternCard(pattern: TripPattern) -> impl IntoView {
    let duration_mins = pattern.duration / 60;
    let start = format_time(&pattern.start_time);
    let end = format_time(&pattern.end_time);

    let expanded = RwSignal::new(false);
    let full_trip = expect_context::<FullTripViewSignal>();

    // Clone pattern for the detail view (only rendered when expanded)
    let pattern_for_detail = StoredValue::new(pattern.clone());
    let pattern_for_full = StoredValue::new(pattern.clone());

    view! {
        <div class="trip-pattern" class:trip-pattern--expanded=move || expanded.get()>
            <div class="trip-header trip-header--clickable">
                <span class="trip-times" on:click=move |_| expanded.update(|e| *e = !*e)>
                    {start}" → "{end}
                </span>
                <div class="trip-header-right">
                    <span class="trip-duration">{duration_mins}" min"</span>
                    <button
                        class="trip-fullview-btn"
                        on:click=move |_| full_trip.0.set(Some(pattern_for_full.get_value()))
                        title="Åpne fullvisning"
                    >
                        "⛶"
                    </button>
                    <span
                        class="trip-expand-icon"
                        on:click=move |_| expanded.update(|e| *e = !*e)
                    >
                        {move || if expanded.get() { "▲" } else { "▼" }}
                    </span>
                </div>
            </div>
            <div class="trip-legs" on:click=move |_| expanded.update(|e| *e = !*e)>
                {pattern
                    .legs
                    .into_iter()
                    .map(|leg| {
                        view! { <LegView leg=leg /> }
                    })
                    .collect_view()}
            </div>
            {move || {
                if expanded.get() {
                    let p = pattern_for_detail.get_value();
                    Some(view! {
                        <div class="trip-detail-section">
                            <RouteDetailView pattern=p />
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
fn LegView(leg: Leg) -> impl IntoView {
    let mode_icon = leg.mode.to_string();
    let time_start = format_time(&leg.expected_start);
    let time_end = format_time(&leg.expected_end);

    let delayed_start = is_delayed(&leg.aimed_start, &leg.expected_start);
    let delayed_end = is_delayed(&leg.aimed_end, &leg.expected_end);
    let realtime_start = leg.aimed_start.is_some();
    let realtime_end = leg.aimed_end.is_some();

    let aimed_start_str = if delayed_start {
        leg.aimed_start.as_ref().map(|t| format!("({})", format_time(t)))
    } else { None };
    let aimed_end_str = if delayed_end {
        leg.aimed_end.as_ref().map(|t| format!("({})", format_time(t)))
    } else { None };

    let line_info = leg
        .line
        .as_ref()
        .map(|l| format!("{} {}", l.public_code, leg.destination.as_deref().unwrap_or(&l.name)))
        .unwrap_or_default();

    let from_name = leg.from_name.clone();
    let to_name = leg.to_name.clone();

    view! {
        <div class="leg">
            <div class="leg-mode">{mode_icon}</div>
            <div class="leg-details">
                <div class="leg-line">{line_info}</div>
                <div class="leg-stops">
                    <span class="leg-from">
                        <span class:delayed=delayed_start class:realtime=realtime_start>{time_start}</span>
                        {aimed_start_str.map(|s| view! { <span class="aimed-time">" "{s}</span> })}
                        " "{from_name}
                    </span>
                    <span class="leg-arrow">" → "</span>
                    <span class="leg-to">
                        <span class:delayed=delayed_end class:realtime=realtime_end>{time_end}</span>
                        {aimed_end_str.map(|s| view! { <span class="aimed-time">" "{s}</span> })}
                        " "{to_name}
                    </span>
                </div>
            </div>
        </div>
    }
}

fn format_time(iso: &str) -> String {
    // Extract HH:MM from ISO-8601 datetime
    if let Some(t_idx) = iso.find('T') {
        let time_part = &iso[t_idx + 1..];
        if time_part.len() >= 5 {
            return time_part[..5].to_string();
        }
    }
    iso.to_string()
}

fn is_delayed(aimed: &Option<String>, expected: &str) -> bool {
    match aimed {
        Some(aimed) => aimed != expected,
        None => false,
    }
}
