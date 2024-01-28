//! Properties editor for [`Project`]s.
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::project;
use std::rc::Rc;
use thot_core::project::Project;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// ************************
// *** Properties State ***
// ************************
enum ProjectPropertiesStateAction {
    SetName(String),
    SetDescription(String),
    Update(Project),
}

#[derive(PartialEq, Clone, Debug)]
struct ProjectProperties {
    pub name: String,
    pub description: Option<String>,
}

impl Reducible for ProjectProperties {
    type Action = ProjectPropertiesStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            ProjectPropertiesStateAction::SetName(value) => {
                current.name = value;
            }

            ProjectPropertiesStateAction::SetDescription(value) => {
                let value = value.trim();
                if value.is_empty() {
                    current.description = None;
                } else {
                    let _ = current.description.insert(value.to_string());
                }
            }

            ProjectPropertiesStateAction::Update(project) => {
                return Self::from(project).into();
            }
        }

        current.into()
    }
}

impl From<Project> for ProjectProperties {
    fn from(value: Project) -> Self {
        Self {
            name: value.name,
            description: value.description,
        }
    }
}

// ****************************
// *** Properties Component ***
// ****************************

/// Properties for [`ProjectEditor`].
#[derive(PartialEq, Properties)]
pub struct ProjectEditorProps {
    pub project: Project,
}

#[function_component(ProjectEditor)]
pub fn project_editor(props: &ProjectEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let onchange = use_callback(props.project.clone(), {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.dispatcher();
        move |properties: ProjectProperties, project| {
            let mut project = project.clone();
            project.name = properties.name.clone();
            project.description = properties.description.clone();

            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            spawn_local(async move {
                match project::update_project(project.clone()).await {
                    Ok(_) => {
                        projects_state.dispatch(ProjectsStateAction::UpdateProject(project.clone()))
                    }
                    Err(err) => {
                        let mut msg = Message::error("Could not update project");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        }
    });

    html! {
        <ProjectEditorView project={props.project.clone()} {onchange} />
    }
}

/// Properties for [`ProjectEditorView`].
#[derive(PartialEq, Properties)]
pub struct ProjectEditorViewProps {
    project: Project,

    #[prop_or_default]
    onchange: Callback<ProjectProperties>,
}

#[function_component(ProjectEditorView)]
fn project_editor_view(props: &ProjectEditorViewProps) -> Html {
    let properties_state = use_reducer(|| Into::<ProjectProperties>::into(props.project.clone()));

    let dirty_state = use_state(|| false);
    let name_ref = use_node_ref();
    let description_ref = use_node_ref();

    use_effect_with(props.project.clone(), {
        let properties_state = properties_state.dispatcher();
        let dirty_state = dirty_state.setter();

        move |project| {
            dirty_state.set(false);
            properties_state.dispatch(ProjectPropertiesStateAction::Update(project.clone()));
        }
    });

    let onchange_name = use_callback((), {
        let properties_state = properties_state.dispatcher();
        let dirty_state = dirty_state.setter();
        let elm = name_ref.clone();

        move |_: Event, _| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            properties_state.dispatch(ProjectPropertiesStateAction::SetName(value));
            dirty_state.set(true);
        }
    });

    let onchange_description = use_callback((), {
        let properties_state = properties_state.dispatcher();
        let dirty_state = dirty_state.setter();
        let elm = description_ref.clone();

        move |_: Event, _| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            properties_state.dispatch(ProjectPropertiesStateAction::SetDescription(value));
            dirty_state.set(true);
        }
    });

    let onsubmit = use_callback((), move |e: SubmitEvent, _| {
        e.prevent_default();
    });

    use_effect_with(
        (
            properties_state.clone(),
            (*dirty_state).clone(),
            props.onchange.clone(),
        ),
        {
            move |(properties_state, dirty_state, onchange)| {
                if !dirty_state {
                    return;
                }

                onchange.emit((**properties_state).clone());
            }
        },
    );

    html! {
        <div class={"project-editor"}>
            <div class={"thot-ui-editor"}>
                <form class={"thot-ui-project-properties-editor"}
                    {onsubmit} >

                    <div class={"form-field name"}>
                        <label>
                            <h3>{ "Name" }</h3>
                            <input
                                ref={name_ref}
                                placeholder={"(no name)"}
                                min={"1"}
                                value={properties_state.name.clone()}
                                onchange={onchange_name} />
                        </label>
                    </div>

                    <div class={"form-field description"}>
                        <label>
                            <h3>{ "Description" }</h3>
                            <textarea
                                ref={description_ref}
                                placeholder={"(no description)"}
                                value={properties_state.description.clone().unwrap_or("".into())}
                                onchange={onchange_description}></textarea>
                        </label>
                    </div>
                </form>
            </div>
        </div>
    }
}
