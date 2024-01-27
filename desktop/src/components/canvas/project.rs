//! Project component with suspense.
use super::container::ContainerTree;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer, GraphStateReducer};
use thot_core::types::ResourceId;
use thot_ui::widgets::suspense::Loading;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct ProjectProps {
    pub rid: ResourceId,
}

#[tracing::instrument]
#[function_component(Project)]
pub fn project(props: &ProjectProps) -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let project_ref = use_node_ref();

    let clear_selection = {
        let canvas_state = canvas_state.clone();

        Callback::from(move |_: MouseEvent| {
            canvas_state.dispatch(CanvasStateAction::ClearSelected);
        })
    };

    let container_tree_fallback = html! { <Loading text={"Loading container tree"} /> };
    html! {
    <div ref={project_ref}
        class={"project"}
        onclick={clear_selection} >

        <div class={"content"}>
            <div class={"container-tree"}>
                <Suspense fallback={container_tree_fallback}>
                    <ContainerTree root={graph_state.graph.root().clone()} />
                </Suspense>
            </div>
        </div>
    </div>
    }
}
