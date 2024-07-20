use leptos::*;
use std::str::FromStr;

/// `<input type="number" ... /> wrapper.
/// Handles `step` and validation UI.
#[component]
pub fn InputNumber(
    /// Read signal.
    /// Attached to `prop:value`.
    #[prop(into)]
    value: Signal<String>,

    #[prop(into)] oninput: Callback<String>,

    #[prop(optional)] min: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(default = false)] required: bool,
) -> impl IntoView {
    let step = move || {
        value.with(|value| match value.split_once('.') {
            None => 1_f64,
            Some((_, decs)) => 10_f64.powi(-(decs.len() as i32)),
        })
    };

    let is_invalid = move || {
        value
            .with(|value| serde_json::Number::from_str(value))
            .is_err()
    };

    view! {
        <input
            type="number"
            class=("error", is_invalid)
            prop:value=value
            min=min
            max=max
            step=step
            on:input=move |e| oninput(event_target_value(&e))
            required=required
        />
    }
}
