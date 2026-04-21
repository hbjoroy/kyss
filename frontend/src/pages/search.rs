use kyss_shared::{JourneyRequest, JourneyResult, Stop};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use wasm_bindgen::JsCast;

use crate::components::journey_results::JourneyResults;
use crate::components::stop_search::StopSearch;
use crate::components::time_picker::TimePicker;

const API_BASE: &str = "/api";

#[component]
pub fn SearchPage() -> impl IntoView {
    let query_map = use_query_map();

    let initial_from_name = query_map.read().get("from_name").unwrap_or_default();
    let initial_to_name = query_map.read().get("to_name").unwrap_or_default();
    let initial_from_id = query_map.read().get("from").unwrap_or_default();
    let initial_to_id = query_map.read().get("to").unwrap_or_default();

    let from_stop = RwSignal::new(if !initial_from_id.is_empty() {
        Some(Stop {
            id: initial_from_id.clone(),
            name: initial_from_name.clone(),
            lat: 0.0, lon: 0.0, quay_id: None, category: None,
        })
    } else { None });

    let to_stop = RwSignal::new(if !initial_to_id.is_empty() {
        Some(Stop {
            id: initial_to_id.clone(),
            name: initial_to_name.clone(),
            lat: 0.0, lon: 0.0, quay_id: None, category: None,
        })
    } else { None });

    let selected_time = RwSignal::new(Option::<String>::None);
    let results = RwSignal::new(Option::<JourneyResult>::None);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let auto_search = !initial_from_id.is_empty() && !initial_to_id.is_empty();

    let do_search = move || {
        let from = from_stop.get_untracked();
        let to = to_stop.get_untracked();
        let time = selected_time.get_untracked();
        if let (Some(from), Some(to)) = (from, to) {
            loading.set(true);
            error.set(None);
            results.set(None);
            spawn_local(async move {
                match fetch_journey(&from.id, &to.id, time.as_deref()).await {
                    Ok(result) => results.set(Some(result)),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        }
    };

    if auto_search { do_search(); }

    let search_click = move |_| { do_search(); };

    let on_from_select = Callback::new(move |stop: Stop| { from_stop.set(Some(stop)); });
    let on_to_select = Callback::new(move |stop: Stop| { to_stop.set(Some(stop)); });
    let can_search = move || from_stop.get().is_some() && to_stop.get().is_some();

    view! {
        <div class="search-page">
            <div class="search-form">
                <StopSearch label="Fra" on_select=on_from_select initial_value=initial_from_name />
                <StopSearch label="Til" on_select=on_to_select initial_value=initial_to_name />
                <TimePicker selected_time=selected_time />
                <button
                    class="btn btn-primary search-btn"
                    on:click=search_click
                    disabled=move || !can_search() || loading.get()
                >
                    {move || if loading.get() { "Søker..." } else { "Søk reise" }}
                </button>
            </div>

            {move || {
                if let Some(err) = error.get() {
                    view! { <div class="error-message"><p>"Feil: "{err}</p></div> }.into_any()
                } else if let Some(result) = results.get() {
                    view! { <JourneyResults result=result /> }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            <SaveTripTypeSection from_stop=from_stop to_stop=to_stop />
        </div>
    }
}

#[component]
fn SaveTripTypeSection(
    from_stop: RwSignal<Option<Stop>>,
    to_stop: RwSignal<Option<Stop>>,
) -> impl IntoView {
    let show_save = RwSignal::new(false);
    let trip_name = RwSignal::new(String::new());
    let trip_icon = RwSignal::new("🚌".to_string());
    let app_data = expect_context::<crate::storage::AppDataSignal>();

    let save = move |_| {
        let from = from_stop.get();
        let to = to_stop.get();
        let name = trip_name.get();
        if let (Some(from), Some(to)) = (from, to) {
            if !name.is_empty() {
                let tt = kyss_shared::TripType {
                    id: format!("tt-{}", js_sys::Date::now() as u64),
                    name, icon: trip_icon.get(),
                    from_stop: from, to_stop: to,
                    line_preferences: vec![],
                };
                app_data.data.update(|d| { d.trip_types.push(tt); });
                show_save.set(false);
                trip_name.set(String::new());
            }
        }
    };

    let icons = ["🚌", "🏢", "🏠", "👫", "🚶", "🎉", "🛒", "🏋️"];

    view! {
        <div class="save-section">
            <Show when=move || from_stop.get().is_some() && to_stop.get().is_some()>
                <Show
                    when=move || !show_save.get()
                    fallback=move || {
                        view! {
                            <div class="save-form">
                                <input
                                    type="text"
                                    class="save-name-input"
                                    placeholder="Navn på reisetype (f.eks. 'Til jobb')"
                                    prop:value=trip_name
                                    on:input=move |ev| {
                                        let target = ev.target().unwrap();
                                        let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
                                        trip_name.set(input.value());
                                    }
                                />
                                <div class="icon-picker">
                                    {icons.into_iter().map(|icon| {
                                        let icon_str = icon.to_string();
                                        let icon_val = icon_str.clone();
                                        view! {
                                            <button
                                                class="icon-btn"
                                                class:selected=move || trip_icon.get() == icon_str
                                                on:click=move |_| trip_icon.set(icon_val.clone())
                                            >
                                                {icon}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                                <div class="save-actions">
                                    <button class="btn btn-primary" on:click=save>"Lagre"</button>
                                    <button class="btn btn-secondary" on:click=move |_| show_save.set(false)>"Avbryt"</button>
                                </div>
                            </div>
                        }
                    }
                >
                    <button class="btn btn-secondary save-trip-btn" on:click=move |_| show_save.set(true)>
                        "💾 Lagre som reisetype"
                    </button>
                </Show>
            </Show>
        </div>
    }
}

async fn fetch_journey(from_id: &str, to_id: &str, date_time: Option<&str>) -> Result<JourneyResult, String> {
    let window = web_sys::window().unwrap();
    let body = serde_json::to_string(&JourneyRequest {
        from: from_id.to_string(), to: to_id.to_string(),
        date_time: date_time.map(|s| s.to_string()), num_results: Some(5),
    }).map_err(|e| format!("{}", e))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));
    let headers = web_sys::Headers::new().unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(
        &format!("{}/journey", API_BASE), &opts,
    ).map_err(|e| format!("{:?}", e))?;

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await.map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp.dyn_into().map_err(|e| format!("{:?}", e))?;

    if !resp.ok() {
        let text = wasm_bindgen_futures::JsFuture::from(resp.text().unwrap())
            .await.map_err(|e| format!("{:?}", e))?;
        return Err(text.as_string().unwrap_or("Unknown error".to_string()));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().unwrap())
        .await.map_err(|e| format!("{:?}", e))?;
    let result: JourneyResult = serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))?;
    Ok(result)
}
