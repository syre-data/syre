//! Bulk editor for `Container`s.
use super::super::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::container::{
    bulk_update_analysis_associations, bulk_update_properties, BulkUpdatePropertiesArgs,
};
use crate::lib::DisplayName;
use std::collections::HashSet;
use syre_core::project::AnalysisAssociation;
use syre_core::types::{ResourceId, ResourceMap};
use syre_local::types::analysis::AnalysisKind;
use syre_local::types::AnalysisStore;
use syre_local_database::command::container::{
    AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs, PropertiesUpdate,
    RunParametersUpdate,
};
use syre_local_database::command::types::{MetadataAction, TagsAction};
use syre_ui::types::Message;
use syre_ui::widgets::bulk_editor::{
    ContainerPropertiesBulkEditor, RunParametersUpdate as RunParametersUiUpdate,
    ScriptAssociationsBulkEditor, ScriptBulkMap,
};
use syre_ui::widgets::container::analysis_associations::{AddAnalysisAssociation, NameMap};
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
        for (script, params) in container.analyses.iter() {
            if let Some(param_vec) = associations.get_mut(script) {
                param_vec.push(params.clone());
            } else {
                associations.insert(script.clone(), Vec::from([params.clone()]));
            }
        }
    }

    let project_scripts = project_state
        .project_analyses
        .get(&canvas_state.project)
        .expect("`Project`'s `Script`s not loaded");

    let name_map = associations
        .clone()
        .into_keys()
        .map(|assoc| {
            let analysis = project_scripts.get(&assoc).expect("`Script` not found");

            (assoc, analysis.display_name())
        })
        .collect::<NameMap>();

    let remaining_scripts = {
        let num_containers = containers.len();
        project_scripts
            .keys()
            .into_iter()
            .filter_map(|script| {
                let Some(script_containers) = associations.get(&script) else {
                    let name = get_script_name(project_scripts, script);
                    return Some((script.clone(), name));
                };

                if script_containers.len() == num_containers {
                    return None;
                }

                let name = get_script_name(project_scripts, script);
                Some((script.clone(), name))
            })
            .collect::<Vec<_>>()
    };

    // **********************
    // *** event handlers ***
    // **********************

    let onchange_name = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |name| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            update.name = Some(name);

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onchange_kind = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |kind| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            update.kind = Some(kind);

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onchange_description = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |description| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            update.description = Some(description);

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onadd_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |tags| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.insert = tags;
            update.tags = tags_update;

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onremove_tag = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |tag| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            let mut tags_update = TagsAction::default();
            tags_update.remove.push(tag);
            update.tags = tags_update;

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onadd_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onremove_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |key| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.remove.push(key);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
            });
        })
    };

    let onchange_metadata = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let containers = rids.clone();

        Callback::from(move |(key, value)| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let containers = containers.clone();

            let mut update = PropertiesUpdate::default();
            let mut metadata_update = MetadataAction::default();
            metadata_update.insert.insert(key, value);
            update.metadata = metadata_update;

            spawn_local(async move {
                update_properties(containers, update, app_state, graph_state).await;
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
                    c.analyses.keys().cloned().collect::<Vec<ResourceId>>(),
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

            let mut update = AnalysisAssociationBulkUpdate::default();
            update.add.push(AnalysisAssociation::new(script.clone()));

            spawn_local(async move {
                update_script_associations(update_containers, update, app_state, graph_state).await
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
                    c.analyses.keys().cloned().collect::<Vec<ResourceId>>(),
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

            let mut update = AnalysisAssociationBulkUpdate::default();
            update.remove.push(script.clone());
            spawn_local(async move {
                update_script_associations(update_containers, update, app_state, graph_state).await
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
                    c.analyses.keys().cloned().collect::<Vec<ResourceId>>(),
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

            let mut assoc_update = AnalysisAssociationBulkUpdate::default();
            let assoc = RunParametersUpdate {
                analysis: update.script,
                autorun: update.autorun,
                priority: update.priority,
            };

            assoc_update.update.push(assoc);

            spawn_local(async move {
                update_script_associations(update_containers, assoc_update, app_state, graph_state)
                    .await
            });
        })
    };

    html! {
        <div class={"syre-ui-editor px-xl"}>
            <h4 class={"align-center m-0"}>{ "Bulk editor" }</h4>
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

            <AddAnalysisAssociation
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

async fn update_properties(
    containers: Vec<ResourceId>,
    update: PropertiesUpdate,
    app_state: AppStateReducer<'_>,
    graph_state: GraphStateReducer,
) {
    match bulk_update_properties(containers.clone(), update.clone()).await {
        Ok(_) => {
            graph_state.dispatch(GraphStateAction::BulkUpdateContainerProperties(
                BulkUpdatePropertiesArgs {
                    rids: containers,
                    update,
                },
            ));
        }

        Err(err) => {
            let mut msg = Message::error("Could not update Containers");
            msg.set_details(err);
            app_state.dispatch(AppStateAction::AddMessage(msg));
        }
    }
}

async fn update_script_associations(
    containers: Vec<ResourceId>,
    update: AnalysisAssociationBulkUpdate,
    app_state: AppStateReducer<'_>,
    graph_state: GraphStateReducer,
) {
    match bulk_update_analysis_associations(containers.clone(), update.clone()).await {
        Ok(_) => {
            graph_state.dispatch(GraphStateAction::BulkUpdateContainerScriptAssociations(
                BulkUpdateAnalysisAssociationsArgs { containers, update },
            ));
        }

        Err(err) => {
            let mut msg = Message::error("Could not update Container Script Assocations");
            msg.set_details(format!("{err:?}"));
            app_state.dispatch(AppStateAction::AddMessage(msg));
        }
    }
}

fn get_script_name(script_store: &AnalysisStore, script: &ResourceId) -> String {
    match script_store.get(&script).unwrap() {
        AnalysisKind::Script(script) => script
            .name
            .clone()
            .unwrap_or(script.path.to_string_lossy().to_string()),

        AnalysisKind::ExcelTemplate(template) => template
            .name
            .clone()
            .unwrap_or(template.template.path.to_string_lossy().to_string()),
    }
}
