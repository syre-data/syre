//! Edit a [`Container`]'s [`ScriptAssociation`]s.
use crate::commands::container::{
    UpdateScriptAssociationsArgs, UpdateScriptAssociationsStringArgs,
};
use crate::common::invoke;
use crate::components::canvas::{
    CanvasStateReducer, ContainerTreeStateAction, ContainerTreeStateReducer,
};
use crate::hooks::{use_container, use_project_scripts};
use thot_core::project::container::ScriptMap;
use thot_core::project::{RunParameters, Script as CoreScript};
use thot_core::types::ResourceId;
use thot_ui::widgets::container::script_associations::script_associations_editor::NameMap;
use thot_ui::widgets::container::script_associations::{
    AddScriptAssociation, ScriptAssociationsEditor as ContainerScriptsEditor,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ScriptAssociationsEditorProps {
    pub container: ResourceId,

    /// Called after save.
    #[prop_or_default]
    pub onsave: Option<Callback<()>>,
}

#[function_component(ScriptAssociationsEditor)]
pub fn script_associations_editor(props: &ScriptAssociationsEditorProps) -> Html {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeStateReducer` context not found");

    let project_scripts = use_project_scripts(canvas_state.project.clone());

    let container = use_container(props.container.clone());
    let Some(container) = container.as_ref() else {
        panic!("`Container` not loaded");
    };

    let associations = use_state(|| {
        let container = container.lock().expect("could not lock `Container`");
        container.scripts.clone()
    });

    let remaining_scripts = use_state(|| {
        if let Some(project_scripts) = project_scripts.as_ref() {
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
        } else {
            Vec::new()
        }
    });

    {
        let container = container.clone();
        let container = container
            .lock()
            .expect("could not lock `Container`")
            .clone();

        let associations = associations.clone();

        use_effect_with_deps(
            move |container| {
                associations.set(container.scripts.clone());
            },
            container,
        );
    }

    {
        let project_scripts = project_scripts.clone();
        let associations = associations.clone();
        let remaining_scripts = remaining_scripts.clone();

        use_effect_with_deps(
            move |associations| {
                let Some(project_scripts) = project_scripts.as_ref() else {
                    panic!("`Project` `Script`s not loaded");
                };

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

    let name_map = (*associations)
        .clone()
        .into_keys()
        .map(|assoc| {
            let Some(project_scripts) = project_scripts.as_ref() else {
                panic!("`Project` `Script`s not loaded");
            };

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
        let associations = associations.clone();

        Callback::from(move |script: ResourceId| {
            let mut assocs = (*associations).clone();
            assocs.insert(script, RunParameters::new());
            associations.set(assocs);
        })
    };

    let onchange = {
        let associations = associations.clone();

        Callback::from(move |assocs: ScriptMap| {
            associations.set(assocs);
        })
    };

    let onsave = {
        let container = props.container.clone();
        let tree_state = tree_state.clone();
        let associations = associations.clone();
        let onsave = props.onsave.clone();

        Callback::from(move |_: MouseEvent| {
            let container = container.clone();
            let tree_state = tree_state.clone();
            let associations = associations.clone();
            let onsave = onsave.clone();

            spawn_local(async move {
                // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
                // See: https://github.com/tauri-apps/tauri/issues/6078
                let associations_str =
                    serde_json::to_string(&*associations).expect("could not serialize `ScriptMap`");

                let update = UpdateScriptAssociationsStringArgs {
                    rid: container.clone(),
                    associations: associations_str,
                };

                let _res = invoke("update_container_script_associations", update)
                    .await
                    .expect("could not invoke `update_container_script_associations`");

                let update = UpdateScriptAssociationsArgs {
                    rid: container,
                    associations: (*associations).clone(),
                };

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerScriptAssociations(
                    update,
                ));

                if let Some(onsave) = onsave {
                    onsave.emit(());
                }
            });
        })
    };

    html! {
        <div class={classes!("script-associations-editor-widget")}>
            <AddScriptAssociation
                scripts={(*remaining_scripts).clone()}
                {onadd} />

            <ContainerScriptsEditor
                associations={(*associations).clone()}
                {name_map}
                {onchange} />

            <button onclick={onsave}>{ "Save" }</button>
        </div>
    }
}

#[cfg(test)]
#[path = "./script_associations_editor_test.rs"]
mod script_associations_editor_test;
