//! Asset editor.
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::UpdatePropertiesArgs;
use crate::common::invoke;
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::constants::MESSAGE_TIMEOUT;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use thot_ui::widgets::asset::AssetEditor as AssetEditorUi;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetEditorProps {
    pub rid: ResourceId,
}

#[function_component(AssetEditor)]
pub fn asset_editor(props: &AssetEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let container = graph_state
        .asset_map
        .get(&props.rid)
        .expect("`Asset`'s `Container` not found");

    let container = graph_state
        .graph
        .get(container)
        .expect("`Container` not found");

    let asset = container.assets.get(&props.rid).expect("`Asset` not found");
    let onsave = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();

        Callback::from(move |asset: CoreAsset| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();

            spawn_local(async move {
                let Ok(res) = invoke(
                            "update_asset_properties",
                            &UpdatePropertiesArgs {
                                rid: asset.rid.clone(),
                                properties: asset.properties.clone(),
                            },
                        )
                        .await else {
                            app_state.dispatch(AppStateAction::AddMessage(Message::success(
                                "Could not save resource",
                            )));

                            return;
                        };

                graph_state.dispatch(GraphStateAction::UpdateAsset(asset));
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::success("Resource saved"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));
            });
        })
    };

    html! {
        <AssetEditorUi asset={asset.clone()} {onsave} />
    }
}

#[cfg(test)]
#[path = "./asset_editor_test.rs"]
mod asset_editor_test;
