use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::trip_detail_view::{FullTripViewSignal, TripDetailView};
use crate::pages;
use crate::storage::AppDataSignal;

#[component]
pub fn App() -> impl IntoView {
    let app_data = AppDataSignal::new();
    provide_context(app_data);

    let full_trip = FullTripViewSignal(RwSignal::new(None));
    provide_context(full_trip);

    view! {
        <Router>
            <header class="app-header">
                <a href="/" class="logo">"Kyss"</a>
                <span class="tagline">"Finn din neste reise"</span>
            </header>
            <main>
                {move || {
                    if let Some(pattern) = full_trip.0.get() {
                        view! { <TripDetailView pattern=pattern /> }.into_any()
                    } else {
                        view! {
                            <Routes fallback=|| view! { <p>"Side ikke funnet"</p> }>
                                <Route path=path!("/") view=pages::home::HomePage />
                                <Route path=path!("/search") view=pages::search::SearchPage />
                                <Route path=path!("/trip-type/new") view=pages::trip_type::TripTypeNewPage />
                                <Route path=path!("/trip-type/:id") view=pages::trip_type::TripTypeEditPage />
                            </Routes>
                        }.into_any()
                    }
                }}
            </main>
        </Router>
    }
}
