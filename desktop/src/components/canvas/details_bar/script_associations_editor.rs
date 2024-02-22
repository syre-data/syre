//! Edit a [`Container`]'s [`ScriptAssociation`]s.
use crate::app::ProjectsStateReducer;
use crate::commands::container::{update_script_associations, UpdateScriptAssociationsArgs};
use crate::components::canvas::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use syre_core::project::container::AnalysisMap;
use syre_core::project::RunParameters;
use syre_core::types::ResourceId;
use syre_local::types::script::{ScriptKind, ScriptStore};
use syre_ui::widgets::container::script_associations::{
    AddScriptAssociation, NameMap, ScriptAssociationsEditor as ContainerScriptsEditor,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct ScriptAssociationsEditorProps {
    pub container: ResourceId,

    /// Called after save.
    #[prop_or_default]
    pub onsave: Option<Callback<()>>,
}

#[tracing::instrument]
#[function_component(ScriptAssociationsEditor)]
pub fn script_associations_editor(props: &ScriptAssociationsEditorProps) -> HtmlResult {
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let container = graph_state
        .graph
        .get(&props.container)
        .expect("`Container not found");

    let dirty_state = use_state(|| false); // track if changes come from user or are internal
    let associations = use_state(|| container.analyses.clone());

    let project_scripts = use_state(|| {
        projects_state
            .project_scripts
            .get(&canvas_state.project)
            .expect("`Project`'s `Scripts` not loaded")
            .clone()
    });

    let remaining_scripts = use_state(|| {
        project_scripts
            .keys()
            .into_iter()
            .filter_map(|script| {
                if associations.contains_key(&script) {
                    None
                } else {
                    Some((script.clone(), get_script_name(&project_scripts, script)))
                }
            })
            .collect::<Vec<_>>()
    });

    use_effect_with(projects_state.clone(), {
        let project_scripts = project_scripts.setter();
        move |projects_state| {
            project_scripts.set(
                projects_state
                    .project_scripts
                    .get(&canvas_state.project)
                    .expect("`Project`'s `Scripts` not loaded")
                    .clone(),
            );
        }
    });

    use_effect_with((project_scripts.clone(), associations.clone()), {
        let remaining_scripts = remaining_scripts.setter();
        move |(project_scripts, associations)| {
            remaining_scripts.set(
                project_scripts
                    .keys()
                    .into_iter()
                    .filter_map(|script| {
                        if associations.contains_key(script) {
                            None
                        } else {
                            Some((script.clone(), get_script_name(&project_scripts, script)))
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        }
    });

    // Update associations based on container
    use_effect_with(container.clone(), {
        let dirty_state = dirty_state.setter();
        let associations = associations.setter();

        move |container| {
            associations.set(container.analyses.clone());
            dirty_state.set(false);
        }
    });

    // Update remaining scripts based on associations
    use_effect_with((associations.clone(), project_scripts.clone()), {
        let remaining_scripts = remaining_scripts.setter();
        move |(associations, project_scripts)| {
            let scripts = project_scripts
                .keys()
                .into_iter()
                .filter_map(|script| {
                    if associations.contains_key(script) {
                        None
                    } else {
                        Some((script.clone(), get_script_name(&project_scripts, script)))
                    }
                })
                .collect::<Vec<_>>();

            remaining_scripts.set(scripts);
        }
    });

    // Save associations on change
    use_effect_with(
        (
            props.container.clone(),
            associations.clone(),
            dirty_state.clone(),
        ),
        {
            let graph_state = graph_state.dispatcher();
            move |(container, associations, dirty_state)| {
                if !**dirty_state {
                    return;
                }

                let container = container.clone();
                let graph_state = graph_state.clone();
                let associations = associations.clone();

                spawn_local(async move {
                    match update_script_associations(container.clone(), (*associations).clone())
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::debug!(?err);
                            panic!("{err:?}");
                        }
                    }

                    let update = UpdateScriptAssociationsArgs {
                        rid: container,
                        associations: (*associations).clone(),
                    };

                    graph_state
                        .dispatch(GraphStateAction::UpdateContainerScriptAssociations(update));
                });
            }
        },
    );

    let name_map = (*associations)
        .clone()
        .into_keys()
        .filter_map(|assoc| {
            if let Some(script) = project_scripts.get_script(&assoc) {
                let name = match script.name.clone() {
                    Some(name) => name,
                    None => script.path.to_string_lossy().to_string(),
                };

                return Some((assoc, name));
            }

            if let Some(template) = project_scripts.get_excel_template(&assoc) {
                let name = match template.name.clone() {
                    Some(name) => name,
                    None => template.template.path.to_string_lossy().to_string(),
                };

                return Some((assoc, name));
            }

            None
        })
        .collect::<NameMap>();

    let onadd = {
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        Callback::from(move |script: ResourceId| {
            let mut assocs = (*associations).clone();
            assocs.insert(script, RunParameters::new());
            associations.set(assocs);
            dirty_state.set(true);
        })
    };

    let onchange = {
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        Callback::from(move |assocs: AnalysisMap| {
            associations.set(assocs);
            dirty_state.set(true);
        })
    };

    Ok(html! {
        <div class={classes!("script-associations-editor-widget")}>
            <AddScriptAssociation
                scripts={(*remaining_scripts).clone()}
                {onadd} />

            <ContainerScriptsEditor
                associations={(*associations).clone()}
                {name_map}
                {onchange} />
        </div>
    })
}

fn get_script_name(script_store: &ScriptStore, script: &ResourceId) -> String {
    match script_store.get(&script).unwrap() {
        ScriptKind::Script(script) => script
            .name
            .clone()
            .unwrap_or(script.path.to_string_lossy().to_string()),

        ScriptKind::ExcelTemplate(template) => template
            .name
            .clone()
            .unwrap_or(template.template.path.to_string_lossy().to_string()),
    }
}
