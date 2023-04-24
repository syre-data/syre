//! UI for a `Container`.
use crate::components::form::{InlineInput, InlineTextarea};
use crate::widgets::metadata::MetadataEditor;
use crate::widgets::TagsEditor;
use std::rc::Rc;
use thot_core::project::Container as CoreContainer;
use yew::prelude::*;

// ***********************
// *** Container State ***
// ***********************

enum ContainerStateAction {}

#[derive(Debug, Clone, PartialEq)]
struct ContainerState {
    /// The [`Container`](CoreContainer) being represented.
    pub container: Rc<CoreContainer>,
}

impl ContainerState {
    pub fn new(container: Rc<CoreContainer>) -> Self {
        ContainerState { container }
    }
}

impl Reducible for ContainerState {
    type Action = ContainerStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {}
    }
}

type ContainerReducer = UseReducerHandle<ContainerState>;

// ***************************
// *** Container Component ***
// ***************************

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub container: Rc<CoreContainer>,
}

#[function_component(Container)]
pub fn container(props: &ContainerProps) -> Html {
    let container_state = use_reducer(|| ContainerState::new(props.container.clone()));

    html! {
        <div class={classes!("container")}>
            <InlineInput<String> placeholder={"Name"} value={(*props.container).properties.name.clone()}>
                <h2>{ "(no name)" }</h2>
            </InlineInput<String>>

            <InlineInput<String> placeholder={"Type"} value={(*props.container).properties.kind.clone()}>
                <h2>{ "(no type)" }</h2>
            </InlineInput<String>>

            <InlineTextarea placeholder={"Description"}>
                <h2>{ "(no description)" }</h2>
            </InlineTextarea>

            <TagsEditor>
            </TagsEditor>

            <MetadataEditor >
            </MetadataEditor>
        </div>
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
