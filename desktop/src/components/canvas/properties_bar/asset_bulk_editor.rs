//! Bulk editor for `Asset`s.
use super::super::{GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::asset::bulk_update_properties;
use std::collections::HashSet;
use syre_core::types::ResourceId;
use syre_local_database::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use syre_local_database::command::types::{MetadataAction, TagsAction};
use syre_ui::types::Message;
use syre_ui::widgets::bulk_editor::AssetPropertiesBulkEditor;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetBulkEditorProps {
    pub assets: HashSet<ResourceId>,
}

#[function_component(AssetBulkEditor)]
pub fn container_bulk_editor(props: &AssetBulkEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let assets = props
        .assets
        .iter()
        .map(|rid| {
            let container = graph_state.asset_map.get(rid).unwrap();

            let container = graph_state.graph.get(container).unwrap();

            container.assets.get(rid).unwrap()
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

    let onchange_name = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |name| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            update.name = Some(name);

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onchange_kind = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |kind| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            update.kind = Some(kind);

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onchange_description = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |description| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            update.description = Some(description);

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onadd_tag = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onremove_tag = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onadd_metadata = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onremove_metadata = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |key| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    let onchange_metadata = Callback::from({
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let assets = rids.clone();

        move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let assets = assets.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(assets, update, app_state, graph_state).await;
            });
        }
    });

    html! {
        <div class={"syre-ui-editor px-xl"}>
            <h4 class={classes!("align-center", "m-0")}>{ "Bulk editor" }</h4>
            <AssetPropertiesBulkEditor
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

async fn update_properties(
    assets: Vec<ResourceId>,
    update: PropertiesUpdate,
    app_state: AppStateReducer<'_>,
    graph_state: GraphStateReducer,
) {
    match bulk_update_properties(assets.clone(), update.clone()).await {
        Ok(_) => {
            let update = BulkUpdatePropertiesArgs {
                rids: assets,
                update,
            };
            graph_state.dispatch(GraphStateAction::BulkUpdateAssetProperties(update));
        }

        Err(err) => {
            let mut msg = Message::error("Could not update Assets");
            msg.set_details(err);
            app_state.dispatch(AppStateAction::AddMessage(msg));
        }
    }
}
