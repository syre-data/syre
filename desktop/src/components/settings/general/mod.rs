//! General settings.
use yew::prelude::*;

#[function_component(GeneralSettings)]
pub fn general_settings() -> Html {
    html! {
        <h2>{ "General" }</h2>
    }
}

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
