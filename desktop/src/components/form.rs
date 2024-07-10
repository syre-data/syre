use leptos::*;

#[component]
pub fn InputDebounced(
    /// Initial value.
    #[prop(optional)]
    value: Option<String>,

    #[prop(default = "text")] r#type: &'static str,
    debounce: f64,

    #[prop(into)] oninput: Callback<String>,
) -> impl IntoView {
    let (value, set_value) = create_signal(value.unwrap_or(String::new()));
    let value = leptos_use::signal_debounced(value, debounce);
    create_effect(move |_| {
        oninput(value.get());
    });

    view! { <input type=r#type prop:value=value on:input=move |e| set_value(event_target_value(&e))/> }
}
