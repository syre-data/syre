//! [`Asset`](thot_core::project::Asset) editor.
use crate::widgets::StandardPropertiesEditor;
use thot_core::project::Asset as CoreAsset;
use yew::prelude::*;

/// Properties for [`AssetEditor`].
#[derive(Properties, PartialEq)]
pub struct AssetEditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub asset: CoreAsset,

    #[prop_or_default]
    pub onsave: Callback<CoreAsset>,
}

/// [`Asset`](thot_core::project::Asset)s editor.
#[function_component(AssetEditor)]
pub fn asset_editor(props: &AssetEditorProps) -> Html {
    let asset = use_state(|| props.asset.clone());
    {
        let asset = asset.clone();

        use_effect_with_deps(
            move |a| {
                asset.set(a.clone());
            },
            props.asset.clone(),
        );
    }

    let onchange = {
        let asset = asset.clone();

        Callback::from(move |properties| {
            let mut update = (*asset).clone();
            update.properties = properties;
            asset.set(update);
        })
    };

    let onsave = {
        let asset = asset.clone();
        let onsave = props.onsave.clone();

        Callback::from(move |_: MouseEvent| {
            onsave.emit((*asset).clone());
        })
    };

    let class = classes!("thot-ui-asset-editor", props.class.clone());
    html! {
        <div key={asset.rid.clone()} {class}>
            <StandardPropertiesEditor
                properties={asset.properties.clone()}
                {onchange} />

            <div>
                { asset.path.as_path().to_str() }
            </div>
            <div>
                <button onclick={onsave}>{ "Save" }</button>
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./editor_test.rs"]
mod editor_test;
