use kyss_shared::TripType;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::trip_type_card::TripTypeCard;
use crate::storage::AppDataSignal;

#[component]
pub fn HomePage() -> impl IntoView {
    let app_data = expect_context::<AppDataSignal>();
    let navigate = use_navigate();

    let nav_search = navigate.clone();
    let go_to_search = move |_| {
        nav_search("/search", Default::default());
    };

    let nav_new = navigate.clone();
    let go_to_new_trip_type = move |_| {
        nav_new("/trip-type/new", Default::default());
    };

    let on_search = {
        let navigate = navigate.clone();
        Callback::new(move |tt: TripType| {
            let url = format!(
                "/search?from={}&to={}&from_name={}&to_name={}",
                urlencoding::encode(&tt.from_stop.id),
                urlencoding::encode(&tt.to_stop.id),
                urlencoding::encode(&tt.from_stop.name),
                urlencoding::encode(&tt.to_stop.name),
            );
            navigate(&url, Default::default());
        })
    };

    let on_delete = Callback::new(move |id: String| {
        app_data.data.update(|d| {
            d.trip_types.retain(|t| t.id != id);
        });
    });

    view! {
        <div class="home-page">
            <div class="quick-actions">
                <button class="btn btn-primary" on:click=go_to_search>
                    "🔍 Søk reise"
                </button>
                <button class="btn btn-secondary" on:click=go_to_new_trip_type>
                    "+ Ny reisetype"
                </button>
            </div>

            <section class="trip-types-section">
                <h2>"Dine reisetyper"</h2>
                {move || {
                    let types = app_data.data.get().trip_types;
                    if types.is_empty() {
                        view! {
                            <p class="empty-state">
                                "Ingen reisetyper ennå. Søk etter en reise og lagre den!"
                            </p>
                        }
                        .into_any()
                    } else {
                        types
                            .into_iter()
                            .map(|tt| {
                                view! {
                                    <TripTypeCard
                                        trip_type=tt
                                        on_search=on_search
                                        on_delete=on_delete
                                    />
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </section>
        </div>
    }
}
