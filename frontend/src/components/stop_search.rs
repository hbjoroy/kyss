use kyss_shared::Stop;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

const API_BASE: &str = "/api";

#[component]
pub fn StopSearch(
    #[prop(into)] label: String,
    on_select: Callback<Stop>,
    display_value: RwSignal<String>,
) -> impl IntoView {
    let input_value = display_value;
    let suggestions = RwSignal::new(Vec::<Stop>::new());
    let show_suggestions = RwSignal::new(false);
    let debounce_handle = RwSignal::new(Option::<i32>::None);
    let selected = RwSignal::new(false);

    let on_input = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
        let val = input.value();
        input_value.set(val.clone());
        selected.set(false);

        // Cancel previous debounce
        if let Some(handle) = debounce_handle.get_untracked() {
            web_sys::window()
                .unwrap()
                .clear_timeout_with_handle(handle);
        }

        if val.len() < 2 {
            suggestions.set(vec![]);
            show_suggestions.set(false);
            return;
        }

        // Debounce 300ms
        let handle = {
            let val = val.clone();
            let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                spawn_local(async move {
                    match fetch_stops(&val).await {
                        Ok(stops) => {
                            suggestions.set(stops);
                            show_suggestions.set(true);
                        }
                        Err(_) => {
                            suggestions.set(vec![]);
                        }
                    }
                });
            });

            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    300,
                )
                .unwrap()
        };

        debounce_handle.set(Some(handle));
    };

    view! {
        <div class="stop-search">
            <label>{label}</label>
            <input
                type="text"
                class="stop-input"
                prop:value=input_value
                on:input=on_input
                on:focus=move |_| {
                    if !suggestions.get().is_empty() && !selected.get() {
                        show_suggestions.set(true);
                    }
                }
                placeholder="Søk etter holdeplass..."
                autocomplete="off"
            />
            <Show when=move || show_suggestions.get()>
                <ul class="suggestions">
                    {move || {
                        suggestions.get().into_iter().map(|stop| {
                            let name = stop.name.clone();
                            let stop_clone = stop.clone();
                            view! {
                                <li
                                    class="suggestion-item"
                                    on:mousedown=move |_| {
                                        let s = stop_clone.clone();
                                        input_value.set(s.name.clone());
                                        show_suggestions.set(false);
                                        selected.set(true);
                                        on_select.run(s);
                                    }
                                >
                                    <span class="stop-name">{name}</span>
                                </li>
                            }
                        }).collect_view()
                    }}
                </ul>
            </Show>
        </div>
    }
}

async fn fetch_stops(query: &str) -> Result<Vec<Stop>, String> {
    let url = format!("{}/stops/search?q={}", API_BASE, urlencoding::encode(query));

    let window = web_sys::window().unwrap();
    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| format!("{:?}", e))?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|e| format!("{:?}", e))?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().unwrap())
        .await
        .map_err(|e| format!("{:?}", e))?;

    let stops: Vec<Stop> =
        serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))?;

    Ok(stops)
}
