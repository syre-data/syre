//! Main settings component.
//! Contains navigation for other settings.
use super::general::GeneralSettings;
use yew::prelude::*;

#[function_component(Settings)]
pub fn settings() -> Html {
    html! {
        <>
            <GeneralSettings />
        </>
    }
}
