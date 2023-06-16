//! Bulk editor for Containers.
use super::super::{GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::BulkUpdatePropertiesArgs;
use crate::commands::types::{ListAction, StandardPropertiesUpdate};
use crate::common::invoke;
use std::collections::HashSet;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use thot_ui::widgets::bulk_editor::StandardPropertiesBulkEditor;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerBulkEditorProps {
    pub containers: HashSet<ResourceId>,
}

#[function_component(ContainerBulkEditor)]
pub fn container_bulk_editor(props: &ContainerBulkEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let containers = props
        .containers
        .iter()
        .map(|rid| graph_state.graph.get(rid).expect("`Container` not found"))
        .collect::<Vec<_>>();

    let rids = containers.iter().map(|c| c.rid.clone()).collect::<Vec<_>>();
    let properties = containers
        .iter()
        .map(|c| c.properties.clone())
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
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
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
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
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
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
            });
        })
    };

    let onadd_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = StandardPropertiesUpdate::default();
            let mut tags_update = ListAction::default();
            tags_update.add.push(tag);
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
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
            let mut tags_update = ListAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
            });
        })
    };

    let onadd_metadata = {
        let graph_state = graph_state.clone();

        Callback::from(move |(key, value)| {
            tracing::debug!(?key, ?value);
        })
    };

    let onremove_metadata = {
        let graph_state = graph_state.clone();

        Callback::from(move |key| {
            tracing::debug!(?key);
        })
    };

    let onchange_metadata = {
        let graph_state = graph_state.clone();

        Callback::from(move |(key, value)| {
            tracing::debug!(?key, ?value);
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
