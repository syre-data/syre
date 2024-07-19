use std::str::FromStr;

use html::Input;
use leptos::*;

// #[component]
// pub fn InputTextDebounced(
//     debounce: f64,

//     #[prop(into, optional)]
//     value: MaybeSignal<String>,

//     set_value: WriteSignal<String>

//     #[prop(optional)] minlength: Option<usize>,
//     #[prop(default = false)] required: bool,
// ) -> impl IntoView {
//     let (input_value, set_input_value) = create_signal(value.get());
//     let input_value = leptos_use::signal_debounced(input_value, debounce);

//     create_effect(move |_| {
//         set_input_value(value.get())
//     });

//     create_effect(move |_| {
//         oninput(input_value.get());
//     });

//     view! {
//         <input
//             type="text"
//             prop:value=input_value
//             minlength=minlength
//             on:input=move |e| set_input_value(event_target_value(&e))
//             required=required
//         />
//     }
// }

/// `<input type="number" ... /> wrapper.
/// Handles `step` and validation UI.
#[component]
pub fn InputNumber(
    /// Read signal.
    /// Attached to `prop:value`.
    value: ReadSignal<String>,

    /// Write signal.
    /// Attached to `on:input`.
    set_value: WriteSignal<String>,

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
            on:input=move |e| set_value(event_target_value(&e))
            required=required
        />
    }
}

// #[component]
// pub fn InputNumberDebounced(
//     debounce: f64,

//     /// Initial value.
//     #[prop(into, default = MaybeSignal::Static(0.0))]
//     value: MaybeSignal<f64>,

//     #[prop(into)] oninput: Callback<f64>,
//     #[prop(optional)] min: Option<f64>,
//     #[prop(optional)] max: Option<f64>,
//     #[prop(default = false)] required: bool,
// ) -> impl IntoView {
//     let (input_value, set_input_value) = create_signal(value.get().to_string());
//     let input_value = leptos_use::signal_debounced(input_value, debounce);

//     create_effect(move |_| set_input_value(value.get().to_string()));
//     create_effect(move |_| {
//         if let Ok(number) = input_value.with(|value| value.parse()) {
//             oninput(number);
//         }
//     });

//     let step = move || {
//         input_value.with(|value| match value.split_once('.') {
//             None => 1_f64,
//             Some((_, decs)) => 10_f64.powi(-(decs.len() as i32)),
//         })
//     };

//     let is_invalid = move || {
//         input_value
//             .with(|value| serde_json::Number::from_str(value))
//             .is_err()
//     };

//     view! {
//         <input
//             type="number"
//             class=("error", is_invalid)
//             prop:value=input_value
//             min=min
//             max=max
//             step=step
//             on:input=move |e| set_input_value(event_target_value(&e))
//             required=required
//         />
//     }
// }

// #[component]
// pub fn InputCheckboxDebounced(
//     debounce: f64,

//     /// Initial value.
//     #[prop(into, default = MaybeSignal::Static(false))]
//     value: MaybeSignal<bool>,

//     #[prop(into)] onchange: Callback<bool>,
// ) -> impl IntoView {
//     let (input_value, set_input_value) = create_signal(value.get());
//     let input_value = leptos_use::signal_debounced(input_value, debounce);
//     let node_ref = create_node_ref();

//     create_effect(move |_| {
//         set_input_value(value.get());
//     });

//     create_effect(move |_| {
//         onchange(input_value.get());
//     });

//     let change = move |_| {
//         let cb: HtmlElement<Input> = node_ref.get().unwrap();
//         set_input_value(cb.checked());
//     };

//     view! { <input type="checkbox" prop:value=input_value on:change=change ref=node_ref/> }
// }

// #[component]
// pub fn TextareaDebounced(
//     debounce: f64,

//     /// Initial value.
//     #[prop(into, optional)]
//     value: MaybeSignal<String>,

//     #[prop(into)] oninput: Callback<String>,
// ) -> impl IntoView {
//     let (input_value, set_input_value) = create_signal(value());
//     let input_value = leptos_use::signal_debounced(input_value, debounce);

//     create_effect(move |_| {
//         set_input_value(value());
//     });

//     create_effect(move |_| {
//         oninput(input_value());
//     });

//     view! {
//         <textarea prop:value=input_value on:input=move |e| set_input_value(event_target_value(&e))>
//             {input_value.get_untracked()}
//         </textarea>
//     }
// }
