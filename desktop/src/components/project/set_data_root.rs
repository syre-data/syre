//! Set the data root of a project.
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::PathBufArgs;
use crate::commands::project::{GetProjectPathArgs, UpdateProjectArgs};
use crate::common::invoke;
use crate::hooks::use_project;
use serde_wasm_bindgen as swb;
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use thot_core::types::ResourceId;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SetDataRootProps {
    pub project: ResourceId,

    /// Called after the form has been successfully submitted.
    #[prop_or_default]
    pub onsuccess: Option<Callback<()>>,
}

/// Component to set the [`Project`](CoreProject)'s `data_root`.
#[function_component(SetDataRoot)]
pub fn set_data_root(props: &SetDataRootProps) -> Html {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_project(&props.project);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let data_root = use_state(|| None);

    let onclick = {
        let data_root = data_root.clone();
        let prj_id = project.rid.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let data_root = data_root.clone();
            let prj_id = prj_id.clone();

            spawn_local(async move {
                // get project directory
                let prj_path = invoke("get_project_path", GetProjectPathArgs { id: prj_id })
                    .await
                    .expect("could not invoke `get_project_path`");

                let prj_path: PathBuf = swb::from_value(prj_path)
                    .expect("could not convert `get_project_path` result from JsValue");

                // user directory selection
                let path = FileDialogBuilder::new()
                    .set_title("Select data root")
                    .set_default_path(&prj_path)
                    .pick_folder()
                    .await;

                let path = path.expect("could not retrieve directory");
                data_root.set(path);
            });
        })
    };

    let onsubmit = {
        let onsuccess = props.onsuccess.clone();
        let projects_state = projects_state.clone();
        let project = project.clone();
        let data_root = data_root.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let onsuccess = onsuccess.clone();
            let projects_state = projects_state.clone();
            let mut project = project.clone();
            let data_root = data_root.clone();

            if let Some(path) = (*data_root).clone() {
                // initialize data root as container, if needed
                spawn_local(async move {
                    let rid = invoke("init_container", PathBufArgs { path })
                        .await
                        .expect("could not invoke `init_container`");

                    let _rid: ResourceId = swb::from_value(rid)
                        .expect("could not convert `init_container` result from JsValue");
                });
            }

            project.data_root = (*data_root).clone();
            {
                // save project
                let onsuccess = onsuccess.clone();
                let project = project.clone();
                let projects_state = projects_state.clone();

                spawn_local(async move {
                    let res = invoke(
                        "update_project",
                        UpdateProjectArgs {
                            project: project.clone(),
                        },
                    )
                    .await
                    .expect("could not invoke `update_project`");

                    let _res: () = swb::from_value(res)
                        .expect("could not convert `update_project` result from JsValue");

                    projects_state.dispatch(ProjectsStateAction::UpdateProject(project));
                    if let Some(onsuccess) = onsuccess {
                        onsuccess.emit(());
                    }
                });
            }
        })
    };

    html! {
        <form class={classes!("align-center")} {onsubmit}>
            <div>
                if let Some(path) = (*data_root).clone() {
                    <span>{ path.to_str().expect("could not convert path to string") }</span>
                }
                <button type={"button"} {onclick}>
                    if data_root.is_none() {
                        { "Set" }
                    } else {
                        { "Change" }
                    }
                </button>
            </div>
            <div>
                <button disabled={data_root.is_none()}>{ "Save" }</button>
            </div>
        </form>
    }
}

#[cfg(test)]
#[path = "./set_data_root_test.rs"]
mod set_data_root_test;
