use kyss_shared::TripPattern;
use leptos::prelude::*;

use crate::components::route_detail::RouteDetailView;

/// Context signal for the full-screen trip detail view.
/// Set to Some(pattern) to show the full view; None to dismiss.
#[derive(Clone, Copy)]
pub struct FullTripViewSignal(pub RwSignal<Option<TripPattern>>);

/// Full-screen trip detail view that takes over the main content area.
/// This component is the extensible container — add new sections/features here.
#[component]
pub fn TripDetailView(pattern: TripPattern) -> impl IntoView {
    let signal = expect_context::<FullTripViewSignal>();

    let duration_mins = pattern.duration / 60;
    let start = format_time(&pattern.start_time);
    let end = format_time(&pattern.end_time);
    let summary = format!("{} → {}  ·  {} min", start, end, duration_mins);

    view! {
        <div class="trip-detail-fullview">
            <div class="trip-detail-toolbar">
                <button
                    class="trip-detail-close"
                    on:click=move |_| signal.0.set(None)
                    title="Lukk"
                >
                    "✕"
                </button>
                <span class="trip-detail-summary">{summary}</span>
            </div>
            <div class="trip-detail-content">
                <RouteDetailView pattern=pattern />
            </div>
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
