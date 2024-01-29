//! [`Asset`](syre_core::project::Asset) editor.
use super::AssetPropertiesEditor;
use syre_core::project::{Asset, AssetProperties};
use yew::prelude::*;

/// Properties for [`AssetEditor`].
#[derive(Properties, PartialEq)]
pub struct AssetEditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub asset: Asset,

    pub onchange_properties: Callback<AssetProperties>,
}

/// [`Asset`](syre_core::project::Asset)s editor.
#[tracing::instrument(skip(props))]
#[function_component(AssetEditor)]
pub fn asset_editor(props: &AssetEditorProps) -> Html {
    let class = classes!("syre-ui-asset-editor", props.class.clone());
    html! {
        <div key={props.asset.rid.clone()} {class}>
            <AssetPropertiesEditor
                properties={props.asset.properties.clone()}
                onchange={props.onchange_properties.clone()}/>

            <div class={classes!("syre-ui-asset-file_name")}>
                { props.asset.path.as_path().to_str() }
            </div>
        </div>
    }
}
