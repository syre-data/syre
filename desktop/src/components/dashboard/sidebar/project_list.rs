//! Project list.
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::hooks::{use_user, use_user_projects};
use crate::routes::Route;
use thot_core::types::ResourceId;
use yew::prelude::*;
use yew_router::prelude::*;

/// Properties for [`ProjectList`].
#[derive(PartialEq, Properties)]
pub struct ProjectListProps {
    // pub projects: Vec<Rc<Project>>,
}

/// Display project list.
#[function_component(ProjectList)]
pub fn project_list(props: &ProjectListProps) -> HtmlResult {
    let app_state =
        use_context::<ProjectsStateReducer>().expect("`AppStateReducer` context not found");

    let navigator = use_navigator().expect("navigator not found");
    let route = use_route::<Route>().expect("route not found");

    let user = use_user();
    let Some(user) = user.as_ref() else {
        panic!("user not set"); // @todo: Redirect to login.
    };

    tracing::debug!("0a");
    let projects = use_user_projects(&user.rid);
    tracing::debug!("0b");

    // Opens the project and navigates to the workspace if needed.
    let open_and_activate_project = move |rid: ResourceId| -> Callback<MouseEvent> {
        let app_state = app_state.clone();
        let navigator = navigator.clone();
        let route = route.clone();
        let rid = rid.clone();

        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(ProjectsStateAction::AddOpenProject(rid.clone()));
            app_state.dispatch(ProjectsStateAction::SetActiveProject(rid.clone()));

            // go to workspace if not there
            if route != Route::Workspace {
                navigator.push(&Route::Workspace);
            }
        })
    };

    Ok(html! {
        <ol>
            { projects.iter().map(|prj| html! {
                <li key={prj.rid.clone()}>
                     <button class={classes!("btn-link")} onclick={open_and_activate_project(prj.rid.clone())}>{
                        &prj.name
                    }</button>
                </li>
            }).collect::<Html>() }
        </ol>
    })
}
