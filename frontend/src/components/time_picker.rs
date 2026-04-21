use kyss_shared::TimePeriod;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::storage::AppDataSignal;

/// Returns the selected ISO-8601 datetime string, or None for "now".
#[component]
pub fn TimePicker(
    selected_time: RwSignal<Option<String>>,
) -> impl IntoView {
    let app_data = expect_context::<AppDataSignal>();
    let active_period = RwSignal::new(Option::<String>::None);
    let show_adjust = RwSignal::new(false);
    let time_value = RwSignal::new(current_hhmm());

    // "Now" is the default
    let select_now = move |_| {
        selected_time.set(None);
        active_period.set(None);
        show_adjust.set(false);
    };

    let on_time_input = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
        let val = input.value();
        time_value.set(val.clone());
        selected_time.set(Some(hhmm_to_iso(&val)));
        // Clear period highlight if time drifts outside the active period
        if let Some(pid) = active_period.get_untracked() {
            let periods = app_data.data.get_untracked().time_periods;
            if let Some(p) = periods.iter().find(|p| p.id == pid) {
                if !time_in_period(&val, p) {
                    active_period.set(None);
                }
            }
        }
    };

    view! {
        <div class="time-picker">
            <label>"Når"</label>
            <div class="time-periods">
                <button
                    class="period-chip"
                    class:active=move || selected_time.get().is_none()
                    on:click=select_now
                >
                    "⏱️ Nå"
                </button>
                {move || {
                    let periods = app_data.data.get().time_periods;
                    periods.into_iter().map(|p| {
                        let pid = p.id.clone();
                        let pid2 = p.id.clone();
                        let start = p.start.clone();
                        let icon = p.icon.clone();
                        let name = p.name.clone();
                        view! {
                            <button
                                class="period-chip"
                                class:active=move || active_period.get().as_deref() == Some(&pid)
                                on:click=move |_| {
                                    time_value.set(start.clone());
                                    selected_time.set(Some(hhmm_to_iso(&start)));
                                    active_period.set(Some(pid2.clone()));
                                    show_adjust.set(true);
                                }
                                title={format!("{} – {}", p.start, p.end)}
                            >
                                {icon}" "{name}
                            </button>
                        }
                    }).collect_view()
                }}
            </div>

            <Show when=move || show_adjust.get() || selected_time.get().is_some()>
                <div class="time-adjust">
                    <input
                        type="time"
                        class="time-input"
                        prop:value=time_value
                        on:input=on_time_input
                    />
                    {move || {
                        if let Some(pid) = active_period.get() {
                            let periods = app_data.data.get().time_periods;
                            if let Some(p) = periods.iter().find(|p| p.id == pid) {
                                let range = format!("{} – {}", p.start, p.end);
                                return view! {
                                    <span class="period-range">{range}</span>
                                }.into_any();
                            }
                        }
                        view! {}.into_any()
                    }}
                    <button
                        class="time-clear"
                        on:click=select_now
                        title="Bruk nåtid"
                    >
                        "✕"
                    </button>
                </div>
            </Show>
        </div>
    }
}

fn current_hhmm() -> String {
    let date = js_sys::Date::new_0();
    format!(
        "{:02}:{:02}",
        date.get_hours(),
        date.get_minutes()
    )
}

fn hhmm_to_iso(hhmm: &str) -> String {
    let date = js_sys::Date::new_0();
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();

    // Parse HH:MM
    let parts: Vec<&str> = hhmm.split(':').collect();
    let (hour, min) = if parts.len() >= 2 {
        (
            parts[0].parse::<u32>().unwrap_or(0),
            parts[1].parse::<u32>().unwrap_or(0),
        )
    } else {
        (0, 0)
    };

    // If the chosen time has already passed today, use tomorrow
    let now_mins = date.get_hours() * 60 + date.get_minutes();
    let chosen_mins = hour * 60 + min;

    let (y, m, d) = if chosen_mins < now_mins {
        // Use tomorrow
        let tomorrow = js_sys::Date::new_with_year_month_day(
            year, date.get_month() as i32, day as i32 + 1,
        );
        (
            tomorrow.get_full_year(),
            tomorrow.get_month() + 1,
            tomorrow.get_date(),
        )
    } else {
        (year, month, day)
    };

    // Get timezone offset in ±HH:MM format
    let offset_mins = date.get_timezone_offset() as i32;
    let offset_sign = if offset_mins <= 0 { "+" } else { "-" };
    let offset_h = offset_mins.unsigned_abs() / 60;
    let offset_m = offset_mins.unsigned_abs() % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:00{}{:02}:{:02}",
        y, m, d, hour, min, offset_sign, offset_h, offset_m
    )
}

fn time_in_period(hhmm: &str, period: &TimePeriod) -> bool {
    hhmm >= period.start.as_str() && hhmm <= period.end.as_str()
}
