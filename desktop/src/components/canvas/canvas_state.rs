//! State for th eproject workspace;
use crate::components::details_bar::DetailsBarWidget;
use std::rc::Rc;
use thot_core::types::ResourceId;
use yew::prelude::*;

pub enum CanvasStateAction {
    /// Set the active widget in the details bar.
    SetDetailsBarWidget(DetailsBarWidget),

    /// Clear the details bar.
    ClearDetailsBar,
}

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    /// Id of the `Project` the canvas is for.
    pub project: ResourceId,
    pub details_bar_widget: Option<DetailsBarWidget>,
    show_side_bars: UseStateHandle<bool>,
}

impl CanvasState {
    pub fn new(project: ResourceId, show_side_bars: UseStateHandle<bool>) -> Self {
        Self {
            project,
            details_bar_widget: None,
            show_side_bars,
        }
    }
}

impl Reducible for CanvasState {
    type Action = CanvasStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            CanvasStateAction::SetDetailsBarWidget(widget) => {
                current.details_bar_widget = Some(widget);
                current.show_side_bars.set(true);
            }
            CanvasStateAction::ClearDetailsBar => {
                current.details_bar_widget = None;
            }
        }

        current.into()
    }
}

pub type CanvasStateReducer = UseReducerHandle<CanvasState>;

#[cfg(test)]
#[path = "./canvas_state_test.rs"]
mod canvas_state_test;
