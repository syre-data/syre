//! Bulk editor for `Asset`s.
use super::super::{GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::BulkUpdatePropertiesArgs;
use crate::commands::types::{MetadataAction, StandardPropertiesUpdate, TagsAction};
use crate::common::invoke;
use std::collections::HashSet;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use thot_ui::widgets::bulk_editor::StandardPropertiesBulkEditor;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetBulkEditorProps {
    pub assets: HashSet<ResourceId>,
}

#[function_component(AssetBulkEditor)]
pub fn container_bulk_editor(props: &AssetBulkEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let assets = props
        .assets
        .iter()
        .map(|rid| {
            let container = graph_state
                .asset_map
                .get(rid)
                .expect("`Asset`'s `Container` not found");

            let container = graph_state
                .graph
                .get(container)
                .expect("`Container` not found");

            container.assets.get(rid).expect("`Asset` not found")
        })
        .collect::<Vec<_>>();

    let rids = assets.iter().map(|c| c.rid.clone()).collect::<Vec<_>>();
    let properties = assets
        .iter()
        .map(|a| a.properties.clone())
        .collect::<Vec<_>>();

    // **********************
    // *** event handlers ***
    // **********************

    let onchange_name = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |name| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.name = Some(name);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onchange_kind = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |kind| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.kind = Some(kind);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onchange_description = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |description| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.description = Some(description);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onadd_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onremove_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onadd_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onremove_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |key| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    let onchange_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_asset_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
            });
        })
    };

    html! {
        <div class={classes!("thot-ui-editor")}>
            <StandardPropertiesBulkEditor
                {properties}
                {onchange_name}
                {onchange_kind}
                {onchange_description}
                {onadd_tag}
                {onremove_tag}
                {onadd_metadata}
                {onremove_metadata}
                {onchange_metadata} />
        </div>
    }
}
