//! Gets the details bar widget.
use crate::components::canvas::properties_bar::PropertiesBarWidget;
use crate::components::canvas::CanvasStateReducer;
use yew::prelude::*;

#[hook]
pub fn use_properties_bar_widget() -> UseStateHandle<PropertiesBarWidget> {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let properties_bar_widget = use_state(|| canvas_state.properties_bar_widget.clone());
    use_effect_with(canvas_state, {
        let properties_bar_widget = properties_bar_widget.setter();
        move |canvas_state| {
            properties_bar_widget.set(canvas_state.properties_bar_widget.clone());
        }
    });

    properties_bar_widget
}
