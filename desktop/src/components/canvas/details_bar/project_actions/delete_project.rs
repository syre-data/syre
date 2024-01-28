use crate::{
    app::{ProjectsStateAction, ProjectsStateReducer},
    commands::project,
};
use thot_core::project::Project;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::app::{AppStateAction, AppStateReducer};

#[derive(PartialEq, Properties)]
pub struct DeleteProjectProps {
    pub project: Project,
}

#[function_component(DeleteProject)]
pub fn delete_project(props: &DeleteProjectProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let onsubmit = use_callback((), move |e: SubmitEvent, _| {
        e.prevent_default();
    });

    let cancel = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(None));
        }
    });

    let delete = use_callback(props.project.clone(), {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.dispatcher();

        move |_, project| {
            let project = project.clone();
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            spawn_local(async move {
                match project::delete_project(project.rid.clone()).await {
                    Ok(_) => {
                        projects_state
                            .dispatch(ProjectsStateAction::RemoveProject(project.rid.clone()));

                        app_state.dispatch(AppStateAction::SetActiveWidget(None));
                    }

                    Err(err) => {
                        let mut msg =
                            Message::error(format!("Could not delete project {:?}", project.name));
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        }
    });

    html! {
        <div class={"delete-project-widget"}>
            <div>
                <p>{ "Are you sure you want to delete project" }</p>
                <p class={"project-name"}>{ &props.project.name }</p>
            </div>
            <div>
                <form {onsubmit}>
                    <button class={"btn-danger"}
                        onclick={delete}>

                        { "Yes, delete"}
                    </button>

                    <button onclick={cancel}>
                        {"Cancel"}
                    </button>
                </form>
            </div>
        </div>
    }
}
