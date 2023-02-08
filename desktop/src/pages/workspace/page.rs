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

#[cfg(test)]
#[path = "./page_test.rs"]
mod page_test;
