use kyss_shared::Stop;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

use crate::components::stop_search::StopSearch;
use crate::storage::AppDataSignal;

#[component]
pub fn TripTypeNewPage() -> impl IntoView {
    let app_data = expect_context::<AppDataSignal>();
    let navigate = use_navigate();

    let name = RwSignal::new(String::new());
    let icon = RwSignal::new("🚌".to_string());
    let from_stop = RwSignal::new(Option::<Stop>::None);
    let to_stop = RwSignal::new(Option::<Stop>::None);

    let icons = ["🚌", "🏢", "🏠", "👫", "🚶", "🎉", "🛒", "🏋️"];

    let save = move |_| {
        let n = name.get();
        let from = from_stop.get();
        let to = to_stop.get();
        if let (Some(from), Some(to)) = (from, to) {
            if !n.is_empty() {
                let tt = kyss_shared::TripType {
                    id: format!("tt-{}", js_sys::Date::now() as u64),
                    name: n, icon: icon.get(),
                    from_stop: from, to_stop: to,
                    line_preferences: vec![],
                };
                app_data.data.update(|d| { d.trip_types.push(tt); });
                navigate("/", Default::default());
            }
        }
    };

    let on_from = Callback::new(move |stop: Stop| from_stop.set(Some(stop)));
    let on_to = Callback::new(move |stop: Stop| to_stop.set(Some(stop)));

    view! {
        <div class="trip-type-page">
            <h2>"Ny reisetype"</h2>
            <div class="form-group">
                <label>"Navn"</label>
                <input
                    type="text"
                    placeholder="F.eks. 'Til jobb'"
                    prop:value=name
                    on:input=move |ev| {
                        let target = ev.target().unwrap();
                        let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
                        name.set(input.value());
                    }
                />
            </div>
            <div class="form-group">
                <label>"Ikon"</label>
                <div class="icon-picker">
                    {icons.into_iter().map(|i| {
                        let i_str = i.to_string();
                        let i_val = i_str.clone();
                        view! {
                            <button
                                class="icon-btn"
                                class:selected=move || icon.get() == i_str
                                on:click=move |_| icon.set(i_val.clone())
                            >
                                {i}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            <StopSearch label="Fra" on_select=on_from initial_value="" />
            <StopSearch label="Til" on_select=on_to initial_value="" />
            <div class="form-actions">
                <button class="btn btn-primary" on:click=save>"Lagre"</button>
                <a href="/" class="btn btn-secondary">"Avbryt"</a>
            </div>
        </div>
    }
}

#[component]
pub fn TripTypeEditPage() -> impl IntoView {
    view! {
        <div class="trip-type-page">
            <h2>"Rediger reisetype"</h2>
            <p>"Kommer snart..."</p>
            <a href="/" class="btn btn-secondary">"Tilbake"</a>
        </div>
    }
}
