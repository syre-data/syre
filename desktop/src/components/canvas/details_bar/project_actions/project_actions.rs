use super::project_editor::ProjectEditor;
use crate::app::{AppStateAction, AppStateReducer, AppWidget, ProjectsStateReducer};
use crate::components::canvas::CanvasStateReducer;
use thot_core::project::Project;
use yew::prelude::*;

#[function_component(ProjectActions)]
pub fn project_actions() -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let project_state = use_state(|| projects_state.projects.get(&canvas_state.project).cloned());

    use_effect_with((projects_state.clone(), canvas_state.project.clone()), {
        let project_state = project_state.setter();
        move |(projects_state, project)| {
            project_state.set(projects_state.projects.get(project).cloned());
        }
    });

    match &*project_state {
        Some(project) => {
            html! {
                <div class="project-actions px-xl h-100 column">
                    <div class={"grow"}>
                        <h2 class={"title"}>{ "Project" }</h2>
                        <ProjectEditor project={project.clone()}/>
                    </div>

                    <ProjectControls project={project.clone()} />
                </div>
            }
        }

        None => {
            html! {
                <h1>{ "Project not found" }</h1>
            }
        }
    }
}

#[derive(Properties, PartialEq)]
struct ProjectControlProps {
    pub project: Project,
}

#[function_component(ProjectControls)]
fn project_controls(props: &ProjectControlProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();

    let confirm_delete = use_callback(props.project.clone(), {
        let app_state = app_state.dispatcher();
        move |_, project| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::DeleteProject(project.clone()),
            )));
        }
    });

    html! {
        <div class={"project-controls"}>
            <button class={"btn-danger"}
                onclick={confirm_delete}>

                { "Delete" }
            </button>
        </div>
    }
}
