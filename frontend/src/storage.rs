use kyss_shared::AppData;
use leptos::prelude::*;
use web_sys::window;

const STORAGE_KEY: &str = "kyss.data";

#[derive(Clone, Copy)]
pub struct AppDataSignal {
    pub data: RwSignal<AppData>,
}

impl AppDataSignal {
    pub fn new() -> Self {
        let loaded = load_from_storage().unwrap_or_default();
        let data = RwSignal::new(loaded);

        // Auto-save whenever data changes
        Effect::new(move || {
            let current = data.get();
            save_to_storage(&current);
        });

        Self { data }
    }
}

fn load_from_storage() -> Option<AppData> {
    let storage = window()?.local_storage().ok()??;
    let json = storage.get_item(STORAGE_KEY).ok()??;

    match serde_json::from_str::<AppData>(&json) {
        Ok(data) => Some(data),
        Err(_) => {
            // Corrupted data — reset to default
            web_sys::console::warn_1(&"Kyss: corrupted localStorage, resetting".into());
            None
        }
    }
}

fn save_to_storage(data: &AppData) {
    let Some(window) = window() else { return };
    let Some(Ok(Some(storage))) = Some(window.local_storage()) else { return };
    if let Ok(json) = serde_json::to_string(data) {
        let _ = storage.set_item(STORAGE_KEY, &json);
    }
}
