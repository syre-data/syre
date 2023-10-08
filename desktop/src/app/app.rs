//! Main application.
use super::{
    app_state::AppState, auth_state::AuthState, projects_state::ProjectsState, AppStateAction,
    AppStateReducer, AuthStateReducer, ProjectsStateAction, ProjectsStateReducer,
};
use crate::commands::project::LoadUserProjectsArgs;
use crate::common::invoke;
use crate::components::messages::Messages;
use crate::routes::{routes::switch, Route};
use crate::widgets::GlobalWidgets;
use futures::stream::StreamExt;
use thot_core::project::Project;
use thot_local::types::ProjectSettings;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

// *********************
// *** App Component ***
// *********************

#[cfg_attr(doc, aquamarine::aquamarine)]
/// App initialization
///
/// ```mermaid
/// flowchart TD
///      start(Initialize app) --> get_active_user(Get active user)
///      get_active_user -- Set --> set_state(Set state)
///      get_active_user -- Not set --> sign_in(Sign in)
///      set_state --> finish(App initialized)
///      sign_in -- Has account --> set_state
///      sign_in -- New user --> create_account(Create account)
///      create_account --> set_state
/// ```
#[function_component(App)]
pub fn app() -> Html {
    let auth_state = use_reducer(|| AuthState::default());
    let app_state = use_reducer(|| AppState::default());
    let projects_state = use_reducer(|| ProjectsState::default());

    {
        // load user projects
        let auth_state = auth_state.clone();
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();

        use_effect_with(auth_state, move |auth_state| {
            let Some(user) = auth_state.user.as_ref() else {
                return;
            };

            let user_id = user.rid.clone();
            let projects_state = projects_state.clone();

            spawn_local(async move {
                let Ok(projects) = invoke::<Vec<(Project, ProjectSettings)>>(
                    "load_user_projects",
                    LoadUserProjectsArgs { user: user_id },
                )
                .await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not load user projects",
                    )));
                    return;
                };

                projects_state.dispatch(ProjectsStateAction::InsertProjects(projects));
            });
        })
    }

    use_effect_with((), move |_| {
        spawn_local(async move {
            let mut events =
                tauri_sys::event::listen::<thot_local_database::Update>("thot://database-update")
                    .await
                    .expect("could not create `thot://database-update` listener");

            while let Some(event) = events.next().await {
                tracing::debug!(?event);
            }
        });
    });

    // TODO Respond to `open_settings` event.

    html! {
        <BrowserRouter>
        <ContextProvider<AuthStateReducer> context={auth_state.clone()}>
        <ContextProvider<AppStateReducer> context={app_state}>
            if auth_state.is_authenticated() {
                <ContextProvider<ProjectsStateReducer> context={projects_state}>
                    <div id={"content"}>
                        <main>
                            <Switch<Route> render={switch} />
                        </main>
                        <Messages />
                        <GlobalWidgets />
                        <div id={"app-main-shadow-box"}></div>
                    </div>
                </ContextProvider<ProjectsStateReducer>>
            } else {
                <main>
                    <Switch<Route> render={switch} />
                </main>
            }
        </ContextProvider<AppStateReducer>>
        </ContextProvider<AuthStateReducer>>
        </BrowserRouter>
    }
}
