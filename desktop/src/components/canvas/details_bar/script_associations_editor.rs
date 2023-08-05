//! Edit a [`Container`]'s [`ScriptAssociation`]s.
use crate::app::ProjectsStateReducer;
use crate::commands::container::{
    UpdateScriptAssociationsArgs, UpdateScriptAssociationsStringArgs,
};
use crate::common::invoke;
use crate::components::canvas::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use thot_core::project::container::ScriptMap;
use thot_core::project::{RunParameters, Script as CoreScript};
use thot_core::types::ResourceId;
use thot_ui::widgets::container::script_associations::{
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
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let container = graph_state
        .graph
        .get(&props.container)
        .expect("`Container not found");

    let dirty_state = use_state(|| false); // track if changes come from user or are internal
    let associations = use_state(|| container.scripts.clone());

    let project_scripts = projects_state
        .project_scripts
        .get(&canvas_state.project)
        .expect("`Project`'s `Scripts` not loaded");

    let remaining_scripts = use_state(|| {
        project_scripts
            .values()
            .filter_map(|script| {
                if associations.contains_key(&script.rid) {
                    None
                } else {
                    Some(script.clone())
                }
            })
            .collect::<Vec<CoreScript>>()
    });

    {
        // Update associations based on container
        let container = container.clone();
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        use_effect_with_deps(
            move |container| {
                associations.set(container.scripts.clone());
                dirty_state.set(false);
            },
            container,
        );
    }

    {
        // Update remaining scripts based on associations
        let project_scripts = project_scripts.clone();
        let associations = associations.clone();
        let remaining_scripts = remaining_scripts.clone();

        use_effect_with_deps(
            move |associations| {
                let scripts = project_scripts
                    .values()
                    .filter_map(|script| {
                        if associations.contains_key(&script.rid) {
                            None
                        } else {
                            Some(script.clone())
                        }
                    })
                    .collect::<Vec<CoreScript>>();

                remaining_scripts.set(scripts);
            },
            associations,
        );
    }

    {
        // Save associations on change
        let container = props.container.clone();
        let graph_state = graph_state.clone();
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        use_effect_with_deps(
            move |associations| {
                if !*dirty_state {
                    return;
                }

                let container = container.clone();
                let graph_state = graph_state.clone();
                let associations = associations.clone();

                spawn_local(async move {
                    // TODO Issue with deserializing `HashMap` in Tauri, send as string.
                    // See https://github.com/tauri-apps/tauri/issues/6078
                    let associations_str = serde_json::to_string(&*associations)
                        .expect("could not serialize `ScriptMap`");

                    let update = UpdateScriptAssociationsStringArgs {
                        rid: container.clone(),
                        associations: associations_str,
                    };

                    let _res = invoke::<()>("update_container_script_associations", update)
                        .await
                        .expect("could not invoke `update_container_script_associations`");

                    let update = UpdateScriptAssociationsArgs {
                        rid: container,
                        associations: (*associations).clone(),
                    };

                    graph_state
                        .dispatch(GraphStateAction::UpdateContainerScriptAssociations(update));
                });
            },
            associations,
        );
    }

    let name_map = (*associations)
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

        Callback::from(move |assocs: ScriptMap| {
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
