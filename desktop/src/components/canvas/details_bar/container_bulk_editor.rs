//! Bulk editor for `Container`s.
use super::super::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::container::{
    BulkUpdatePropertiesArgs, BulkUpdateScriptAssociationArgs, ContainerPropertiesUpdate,
    RunParametersUpdate, ScriptAssociationsBulkUpdate,
};
use crate::commands::types::{MetadataAction, TagsAction};
use crate::common::invoke;
use std::collections::HashSet;
use thot_core::project::ScriptAssociation;
use thot_core::types::{ResourceId, ResourceMap};
use thot_ui::types::Message;
use thot_ui::widgets::bulk_editor::{
    ContainerPropertiesBulkEditor, RunParametersUpdate as RunParametersUiUpdate,
    ScriptAssociationsBulkEditor, ScriptBulkMap,
};
use thot_ui::widgets::container::script_associations::{AddScriptAssociation, NameMap};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerBulkEditorProps {
    pub containers: HashSet<ResourceId>,
}

#[function_component(ContainerBulkEditor)]
pub fn container_bulk_editor(props: &ContainerBulkEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectStateReducer` context not found");

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

    let mut associations = ScriptBulkMap::new();
    for container in containers.iter() {
        for (script, params) in container.scripts.iter() {
            if let Some(param_vec) = associations.get_mut(script) {
                param_vec.push(params.clone());
            } else {
                associations.insert(script.clone(), Vec::from([params.clone()]));
            }
        }
    }

    let project_scripts = project_state
        .project_scripts
        .get(&canvas_state.project)
        .expect("`Project`'s `Script`s not loaded");

    let name_map = associations
        .clone()
        .into_keys()
        .map(|assoc| {
            let script = project_scripts.get(&assoc).expect("`Script` not found");
            let name = match script.name.clone() {
                Some(name) => name,
                None => {
                    let name = script
                        .path
                        .as_path()
                        .file_name()
                        .expect("could not get path's file name");

                    name.to_str()
                        .expect("could not convert file name to string")
                        .to_string()
                }
            };

            (assoc, name)
        })
        .collect::<NameMap>();

    let remaining_scripts = {
        let num_containers = containers.len();
        project_scripts
            .iter()
            .filter_map(|(sid, script)| {
                let Some(script_containers) = associations.get(&sid) else {
                    return Some(script.clone());
                };

                if script_containers.len() == num_containers {
                    return None;
                }

                Some(script.clone())
            })
            .collect::<Vec<_>>()
    };

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
            let mut update = ContainerPropertiesUpdate::default();
            update.name = Some(name);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
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
            let mut update = ContainerPropertiesUpdate::default();
            update.kind = Some(kind);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
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
            let mut update = ContainerPropertiesUpdate::default();
            update.description = Some(description);
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
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

        Callback::from(move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = ContainerPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
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
            let mut update = ContainerPropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
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
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rids = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let mut update = ContainerPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
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
            let mut update = ContainerPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
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
            let mut update = ContainerPropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;
            let update = BulkUpdatePropertiesArgs {
                rids: rids.clone(),
                update,
            };

            spawn_local(async move {
                let res = invoke::<()>("bulk_update_container_properties", update.clone()).await;
                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Containers",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(update));
            });
        })
    };

    let onadd_association = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_scripts = containers
            .iter()
            .map(|c| {
                (
                    c.rid.clone(),
                    c.scripts.keys().cloned().collect::<Vec<ResourceId>>(),
                )
            })
            .collect::<ResourceMap<_>>();

        Callback::from(move |script: ResourceId| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();

            let update_containers = container_scripts
                .iter()
                .filter_map(|(c, scripts)| {
                    if scripts.contains(&&script) {
                        None
                    } else {
                        Some(c.clone())
                    }
                })
                .collect();

            let mut update = ScriptAssociationsBulkUpdate::default();
            update.add.push(ScriptAssociation::new(script.clone()));
            let update = BulkUpdateScriptAssociationArgs {
                containers: update_containers,
                update,
            };

            spawn_local(async move {
                let res =
                    invoke::<()>("bulk_update_container_script_associations", update.clone()).await;

                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Container Script Assocations",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerScriptAssociations(
                    update,
                ));
            });
        })
    };

    let onremove_association = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_scripts = containers
            .iter()
            .map(|c| {
                (
                    c.rid.clone(),
                    c.scripts.keys().cloned().collect::<Vec<ResourceId>>(),
                )
            })
            .collect::<ResourceMap<_>>();

        Callback::from(move |script: ResourceId| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();

            let update_containers = container_scripts
                .iter()
                .filter_map(|(c, scripts)| {
                    if scripts.contains(&&script) {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let mut update = ScriptAssociationsBulkUpdate::default();
            update.remove.push(script.clone());
            let update = BulkUpdateScriptAssociationArgs {
                containers: update_containers,
                update,
            };

            spawn_local(async move {
                let res =
                    invoke::<()>("bulk_update_container_script_associations", update.clone()).await;

                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Container Script Assocations",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerScriptAssociations(
                    update,
                ));
            });
        })
    };

    let onchange_association = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_scripts = containers
            .iter()
            .map(|c| {
                (
                    c.rid.clone(),
                    c.scripts.keys().cloned().collect::<Vec<ResourceId>>(),
                )
            })
            .collect::<ResourceMap<_>>();

        Callback::from(move |update: RunParametersUiUpdate| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();

            let update_containers = container_scripts
                .iter()
                .filter_map(|(c, scripts)| {
                    if scripts.contains(&update.script) {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let mut assoc_update = ScriptAssociationsBulkUpdate::default();
            let assoc = RunParametersUpdate {
                script: update.script,
                autorun: update.autorun,
                priority: update.priority,
            };

            assoc_update.update.push(assoc);
            let update = BulkUpdateScriptAssociationArgs {
                containers: update_containers,
                update: assoc_update,
            };

            spawn_local(async move {
                let res =
                    invoke::<()>("bulk_update_container_script_associations", update.clone()).await;

                if let Err(_err) = res {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update Container Script Assocations",
                    )));
                    return;
                }

                graph_state.dispatch(GraphStateAction::BulkUpdateContainerScriptAssociations(
                    update,
                ));
            });
        })
    };

    html! {
        <div class={classes!("thot-ui-editor")}>
            <ContainerPropertiesBulkEditor
                {properties}
                {onchange_name}
                {onchange_kind}
                {onchange_description}
                {onadd_tag}
                {onremove_tag}
                {onadd_metadata}
                {onremove_metadata}
                {onchange_metadata} />

            <AddScriptAssociation
                scripts={remaining_scripts}
                onadd={onadd_association} />

            <ScriptAssociationsBulkEditor
                {associations}
                {name_map}
                onremove={onremove_association}
                onchange={onchange_association} />
        </div>
    }
}
