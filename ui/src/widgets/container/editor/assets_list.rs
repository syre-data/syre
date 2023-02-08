//! [`Asset`](thot_core::project::Asset) editor
//! for a [`Container`](thot_core::project::Container).
use thot_core::project::container::AssetMap;
use yew::prelude::*;

/// Properties for [`AssetsList`].
#[derive(Properties, PartialEq)]
pub struct AssetsListProps {
    #[prop_or_default]
    pub class: Classes,
    pub assets: AssetMap,
}

/// [`Asset`](thot_core::project::Asset)s list.
#[function_component(AssetsList)]
pub fn assets_list(props: &AssetsListProps) -> Html {
    html! {
        {"Assets"}
    }
}

#[cfg(test)]
#[path = "./assets_list_test.rs"]
mod assets_list_test;
