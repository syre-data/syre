//! Settings page.
use crate::components::settings::Settings as SettingsUI;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Settings)]
pub fn settings() -> Html {
    html! {
        <SettingsUI />
    }
}

#[cfg(test)]
#[path = "./settings_test.rs"]
mod settings_test;
