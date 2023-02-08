//! Container tree UI.
use super::Container;
use crate::app::AuthStateReducer;
use crate::commands::container::{NewChildArgs, UpdatePropertiesArgs};
use crate::common::invoke;
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use crate::hooks::use_container;
use serde_wasm_bindgen as swb;
use thot_core::project::Container as CoreContainer;
use thot_core::types::{Creator, ResourceId, UserId};
use thot_ui::components::ShadowBox;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// **********************
// *** New Child Name ***
// **********************

/// Properties for a [`NewChildName`].
#[derive(Properties, PartialEq)]
struct NewChildNameProps {
    /// Callback to run on submission.
    #[prop_or_default]
    pub onsubmit: Option<Callback<String>>,
}

/// Component to get name for a new child.
#[function_component(NewChildName)]
fn new_child_name(props: &NewChildNameProps) -> Html {
    let input_ref = use_node_ref();
    let is_input_valid = use_state(|| false);

    let onsubmit = {
        let cb = props.onsubmit.clone();
        let input_ref = input_ref.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            if let Some(cb) = cb.clone() {
                let input = input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("could not cast node ref as input");

                cb.emit(input.value().trim().into());
            }
        })
    };

    let oninput = {
        let input_ref = input_ref.clone();
        let is_input_valid = is_input_valid.clone();

        Callback::from(move |_: web_sys::InputEvent| {
            let input = input_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            is_input_valid.set(!input.value().trim().is_empty());
        })
    };

    html! {
        <form {onsubmit}>
             <input ref={input_ref} {oninput} placeholder="Name" />
             <button disabled={!*is_input_valid}>{ "Add child" }</button>
        </form>
    }
}

// **********************
// *** Container Tree ***
// **********************

/// Properties for a [`ContainerTree`].
#[derive(Properties, PartialEq)]
pub struct ContainerTreeProps {
    /// The root `Container`.
    pub root: ResourceId,
}

/// Container tree component.
#[function_component(ContainerTree)]
pub fn container_tree(props: &ContainerTreeProps) -> HtmlResult {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeReducer` context not found");

    let container = use_container(props.root.clone());
    let Some(container) = container.as_ref() else {
        panic!("`Container` not loaded");
    };

    let show_add_child_form = use_state(|| false);
    let new_child_parent = use_state(|| None);

    // -----------------
    // --- add child ---
    // -----------------

    let onadd_child = {
        let show_add_child_form = show_add_child_form.clone();
        let new_child_parent = new_child_parent.clone();

        Callback::from(move |parent: ResourceId| {
            new_child_parent.set(Some(parent));
            show_add_child_form.set(true);
        })
    };

    let close_add_child = {
        let show_add_child_form = show_add_child_form.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            show_add_child_form.set(false);
        })
    };

    let add_child = {
        let tree_state = tree_state.clone();
        let new_child_parent = new_child_parent.clone();
        let show_add_child_form = show_add_child_form.clone();
        let uid = auth_state
            .user
            .as_ref()
            .expect("`AuthState.user` should be set")
            .rid
            .clone();

        Callback::from(move |name: String| {
            show_add_child_form.set(false);
            let tree_state = tree_state.clone();
            let uid = uid.clone();

            let parent = (*new_child_parent)
                .clone()
                .expect("new child parent not set");

            spawn_local(async move {
                // create child
                let child = invoke(
                    "new_child",
                    NewChildArgs {
                        name,
                        parent: parent.clone(),
                    },
                )
                .await
                .expect("could not invoke `new_child`");

                let mut child: CoreContainer = swb::from_value(child)
                    .expect("could not convert result of `new_child` from JsValue");

                tree_state.dispatch(ContainerTreeStateAction::InsertChildContainer(
                    parent,
                    child.clone(),
                ));

                // set creator
                child.properties.creator = Creator::User(Some(UserId::Id(uid)));

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(
                    UpdatePropertiesArgs {
                        rid: child.rid,
                        properties: child.properties,
                    },
                ));
            });
        })
    };

    // ----------
    // --- ui ---
    // ----------

    let container_fallback = html! { <Loading text={"Loading container"} /> };

    Ok(html! {
        <div class={classes!("container-tree")}>
            <Suspense fallback={container_fallback}>
                <Container
                    rid={props.root.clone()}
                    {onadd_child} />

                <div class={classes!("children")}>
                    { container
                        .lock()
                        .expect("could not lock container")
                        .children
                        .keys()
                        .map(|root| html! {
                            <ContainerTree root={root.clone()} />
                        })
                        .collect::<Html>()
                    }
                </div>
            </Suspense>

            if *show_add_child_form {
                <ShadowBox title="Add child" onclose={close_add_child}>
                    <NewChildName onsubmit={add_child} />
                </ShadowBox>
            }
        </div>
    })
}

#[cfg(test)]
#[path = "./container_tree_test.rs"]
mod container_tree_test;
