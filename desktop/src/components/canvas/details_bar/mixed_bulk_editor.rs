//! Bulk editor for `Asset`s.
use super::super::{GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::asset::bulk_update_properties as bulk_update_asset_properties;
use crate::commands::common::BulkUpdateResourcePropertiesArgs;
use crate::commands::container::bulk_update_properties as bulk_update_container_properties;
use crate::commands::types::ResourcePropertiesUpdate;
use std::collections::HashSet;
use thot_core::project::ResourceProperties;
use thot_core::types::ResourceId;
use thot_local_database::command::types::{MetadataAction, TagsAction};
use thot_ui::types::Message;
use thot_ui::widgets::bulk_editor::ResourcePropertiesBulkEditor;
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
        .map(|c| c.properties.clone().into())
        .collect::<Vec<ResourceProperties>>();

    properties.extend(assets.iter().map(|a| a.properties.clone().into()));

    // **********************
    // *** event handlers ***
    // **********************

    let onchange_name = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |name| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            update.name = Some(name);

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onchange_kind = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |kind| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            update.kind = Some(kind);

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onchange_description = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |description| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            update.description = Some(description);

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onadd_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onremove_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onadd_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onremove_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |key| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    let onchange_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = container_rids.clone();
        let assets = asset_rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();
            let assets = assets.clone();

            let mut update = ResourcePropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_resources(update, containers, assets, graph_state, app_state).await
            });
        })
    };

    html! {
        <div class={classes!("thot-ui-editor")}>
            <h4 class={classes!("align-center", "m-0")}>{ "Bulk editor" }</h4>
            <ResourcePropertiesBulkEditor
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

async fn update_resources(
    update: ResourcePropertiesUpdate,
    containers: Vec<ResourceId>,
    assets: Vec<ResourceId>,
    graph_state: GraphStateReducer,
    app_state: AppStateReducer<'_>,
) {
    match bulk_update_container_properties(containers.clone(), update.clone()).await {
        Ok(_) => {
            graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(
                BulkUpdateResourcePropertiesArgs {
                    rids: containers,
                    update: update.clone(),
                },
            ));
        }
        Err(err) => {
            let mut msg = Message::error("Could not update Containers");
            msg.set_details(err);
            app_state.dispatch(AppStateAction::AddMessage(msg));
            return;
        }
    }

    match bulk_update_asset_properties(assets.clone(), update.clone()).await {
        Ok(_) => {
            graph_state.dispatch(GraphStateAction::BulkUpdateResourceProperties(
                BulkUpdateResourcePropertiesArgs {
                    rids: assets,
                    update: update.clone(),
                },
            ));
        }
        Err(err) => {
            let mut msg = Message::error("Could not update Assets");
            msg.set_details(err);
            app_state.dispatch(AppStateAction::AddMessage(msg));
            return;
        }
    }
}
