#![cfg(target_arch = "wasm32")]
//! Tests for `components/form`.
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use thot_ui::components::form::{InlineInput, InlineTextarea};
use wasm_bindgen_test::*;
use yew::prelude::*;
wasm_bindgen_test_configure!(run_in_browser);

// **********************
// *** InlineTextarea ***
// **********************

#[wasm_bindgen_test]
async fn inline_textarea() {
    #[function_component(App)]
    fn app() -> Html {
        let value = use_state(|| Words(EN, 1..10).fake());
        let onchange = {
            let value = value.clone();

            Callback::from(move |(val, _e): (String, web_sys::MouseEvent)| {
                value.set(val);
            })
        };

        html! {
            <InlineTextarea {onchange} />
        }
    }
}
