//! Fallback for loading components.
use yew::prelude::*;

/// Properties for the [`Loading`] component.
#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    #[prop_or_default]
    pub text: Option<String>,
}

/// Fallback loading component for suspense components.
#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div>
            if let Some(text) = props.text.clone() {
                { text }
            } else {
                { "Loading" }
            }
        </div>
    }
}
