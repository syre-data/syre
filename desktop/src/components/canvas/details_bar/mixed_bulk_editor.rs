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
pub struct MixedBulkEditorProps {
    pub resources: HashSet<ResourceId>,
}

#[function_component(MixedBulkEditor)]
pub fn mixed_bulk_editor(props: &MixedBulkEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let mut containers = Vec::new();
    let mut container_rids = Vec::new();
    let mut assets = Vec::new();
    let mut asset_rids = Vec::new();
    for rid in props.resources.iter() {
        if let Some(container) = graph_state.asset_map.get(rid) {
            let container = graph_state
                .graph
                .get(container)
                .expect("could not retrieve `Container`");

            assets.push(container.assets.get(rid).expect("could not find `Asset`"));
            asset_rids.push(rid.clone());
        } else {
            containers.push(
                graph_state
                    .graph
                    .get(rid)
                    .expect("could not find `Container`"),
            );

            container_rids.push(rid.clone());
        }
    }

    let mut rids = container_rids.clone();
    rids.extend(asset_rids.clone());

    let mut properties = containers
        .iter()
        .map(|c| c.properties.clone())
        .collect::<Vec<_>>();

    properties.extend(assets.iter().map(|a| a.properties.clone()));

    // **********************
    // *** event handlers ***
    // **********************

    let onchange_name = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();

        Callback::from(move |name| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.name = Some(name);

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onchange_kind = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |kind| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.kind = Some(kind);

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onchange_description = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |description| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            update.description = Some(description);

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onadd_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let container_rids = container_rids.clone();
            let asset_rids = asset_rids.clone();
            let rids = rids.clone();

            let mut update = StandardPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onremove_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onadd_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onremove_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |key| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    let onchange_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_rids = container_rids.clone();
        let asset_rids = asset_rids.clone();
        let rids = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            let container_update = BulkUpdatePropertiesArgs {
                rids: container_rids.clone(),
                update: update.clone(),
            };

            let asset_update = BulkUpdatePropertiesArgs {
                rids: asset_rids.clone(),
                update: update.clone(),
            };

            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", container_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                let res = invoke::<()>("bulk_update_asset_properties", asset_update).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Assets",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(update));
            });
        })
    };

    html! {
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
    }
}
