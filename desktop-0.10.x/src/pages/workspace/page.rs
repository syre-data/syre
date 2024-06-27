//! Project workspace page.
use super::workspace::Workspace as WorkspaceComponent;
use crate::navigation::MainNavigation;
use yew::prelude::*;

#[function_component(Workspace)]
pub fn workspace() -> Html {
    html! {
        <>
            <MainNavigation />
            <WorkspaceComponent />
        </>
    }
}
