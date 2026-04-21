use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages;
use crate::storage::AppDataSignal;

#[component]
pub fn App() -> impl IntoView {
    let app_data = AppDataSignal::new();
    provide_context(app_data);

    view! {
        <Router>
            <header class="app-header">
                <a href="/" class="logo">"Kyss"</a>
                <span class="tagline">"Finn din neste reise"</span>
            </header>
            <main>
                <Routes fallback=|| view! { <p>"Side ikke funnet"</p> }>
                    <Route path=path!("/") view=pages::home::HomePage />
                    <Route path=path!("/search") view=pages::search::SearchPage />
                    <Route path=path!("/trip-type/new") view=pages::trip_type::TripTypeNewPage />
                    <Route path=path!("/trip-type/:id") view=pages::trip_type::TripTypeEditPage />
                </Routes>
            </main>
        </Router>
    }
}
