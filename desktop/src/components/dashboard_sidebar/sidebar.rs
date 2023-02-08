//! Sidebar.
use super::{commands::Commands, project_list::ProjectList, script_list::ScriptList};
use crate::app::{AppStateAction, AppStateReducer, AppWidget};
use crate::routes::Route;
use thot_ui::components::navigation::DropdownMenu;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let create_project = {
        let app_state = app_state.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::CreateProject,
            )))
        })
    };

    let project_list_fallback = html! {
        { "Loading projects" }
    };

    html! {
        <div id={"primary-sidebar"} class={classes!("sidebar")}>
            <div class={classes!("header")}>
                <img id={"sidebar-header-logo"} src={"public/logos/logo-white-horizontal.svg"} />
            </div>
            <nav>
                <ul>
                    <li>
                        <DropdownMenu title={"Projects"}>
                            <Suspense fallback={project_list_fallback}>
                                <ProjectList />
                            </Suspense>
                            <button class={classes!("btn-link")} onclick={create_project}>{
                                "+ New Project"
                            }</button>
                        </DropdownMenu>
                    </li>
                    <li>
                        <DropdownMenu title={"Scripts"}>
                            <ScriptList />
                         </DropdownMenu>
                    </li>
                    <li>
                        <Link<Route> to={Route::Dashboard}>{
                            "Dashboard"
                        }</Link<Route>>
                    </li>
                </ul>
            </nav>
            <div id={"sidebar-commands"}>
                <Commands />
            </div>
            <div class={classes!("footer")}>

            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./sidebar_test.rs"]
mod sidebar_test;
