//! Asset editor.
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::asset::UpdatePropertiesStringArgs;
use crate::common::invoke;
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::hooks::use_asset;
use thot_core::project::AssetProperties;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use thot_ui::widgets::asset::AssetEditor as AssetEditorUi;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetEditorProps {
    pub rid: ResourceId,
}

#[tracing::instrument(skip(props))]
#[function_component(AssetEditor)]
pub fn asset_editor(props: &AssetEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");
    let asset = use_asset(&props.rid);

    let onchange_properties = {
        let asset = asset.clone();
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();

        Callback::from(move |properties: AssetProperties| {
            let asset = asset.clone();
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            spawn_local(async move {
                let Ok(_) = invoke::<()>(
                    "update_asset_properties",
                    &UpdatePropertiesStringArgs {
                        rid: asset.rid.clone(),
                        properties: serde_json::to_string(&properties).unwrap(),
                    },
                )
                .await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::success(
                        "Could not save resource",
                    )));

                    return;
                };

                let mut asset = (*asset).clone();
                asset.properties = properties;
                graph_state.dispatch(GraphStateAction::UpdateAsset(asset));
            });
        })
    };

    html! {
        <AssetEditorUi asset={(*asset).clone()} {onchange_properties} />
    }
}
