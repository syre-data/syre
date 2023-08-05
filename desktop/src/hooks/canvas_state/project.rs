//! Gets the canvas' `Project`.
use crate::components::canvas::CanvasStateReducer;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[tracing::instrument]
#[hook]
pub fn use_canvas_project() -> UseStateHandle<ResourceId> {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project = use_state(|| canvas_state.project.clone());

    {
        let canvas_state = canvas_state.clone();
        let project = project.clone();

        use_effect_with_deps(
            move |canvas_state| {
                project.set(canvas_state.project.clone());
            },
            canvas_state,
        );
    };

    project
}
