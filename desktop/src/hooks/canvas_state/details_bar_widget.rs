//! Gets the details bar widget.
use crate::components::canvas::details_bar::DetailsBarWidget;
use crate::components::canvas::CanvasStateReducer;
use yew::prelude::*;

#[hook]
pub fn use_details_bar_widget() -> UseStateHandle<Option<DetailsBarWidget>> {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let details_bar_widget = use_state(|| canvas_state.details_bar_widget.clone());

    {
        let canvas_state = canvas_state.clone();
        let details_bar_widget = details_bar_widget.clone();

        use_effect_with_deps(
            move |canvas_state| {
                details_bar_widget.set(canvas_state.details_bar_widget.clone());
            },
            canvas_state,
        );
    };

    details_bar_widget
}
