use kyss_shared::TripType;
use leptos::prelude::*;

#[component]
pub fn TripTypeCard(
    trip_type: TripType,
    #[prop(into)] on_search: Callback<TripType>,
    #[prop(into)] on_delete: Callback<String>,
) -> impl IntoView {
    let id = trip_type.id.clone();
    let tt_for_search = trip_type.clone();
    let icon = trip_type.icon.clone();
    let name = trip_type.name.clone();
    let route = format!("{} → {}", trip_type.from_stop.name, trip_type.to_stop.name);

    view! {
        <div class="trip-type-card">
            <div
                class="trip-type-main"
                on:click=move |_| on_search.run(tt_for_search.clone())
            >
                <span class="trip-type-icon">{icon.clone()}</span>
                <div class="trip-type-info">
                    <span class="trip-type-name">{name.clone()}</span>
                    <span class="trip-type-route">{route.clone()}</span>
                </div>
            </div>
            <button
                class="trip-type-delete"
                on:click=move |_| on_delete.run(id.clone())
                title="Slett"
            >
                "×"
            </button>
        </div>
    }
}
