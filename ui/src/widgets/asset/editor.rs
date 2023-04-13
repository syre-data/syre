//! [`Asset`](thot_core::project::Asset) editor.
use crate::widgets::StandardPropertiesEditor;
use thot_core::project::{Asset as CoreAsset, StandardProperties};
use yew::prelude::*;

/// Properties for [`AssetEditor`].
#[derive(Properties, PartialEq)]
pub struct AssetEditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub asset: CoreAsset,

    pub onchange_properties: Callback<StandardProperties>,
}

/// [`Asset`](thot_core::project::Asset)s editor.
#[function_component(AssetEditor)]
pub fn asset_editor(props: &AssetEditorProps) -> Html {
    let class = classes!("thot-ui-asset-editor", props.class.clone());
    html! {
        <div key={props.asset.rid.clone()} {class}>
            <StandardPropertiesEditor
                properties={props.asset.properties.clone()}
                onchange={props.onchange_properties.clone()}/>

            <div class={classes!("thot-ui-asset-file_name")}>
                { props.asset.path.as_path().to_str() }
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./editor_test.rs"]
mod editor_test;
