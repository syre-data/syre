//! State for th eproject workspace;
use crate::components::details_bar::DetailsBarWidget;
use std::collections::HashSet;
use std::rc::Rc;
use thot_core::types::ResourceId;
use yew::prelude::*;

pub enum CanvasStateAction {
    /// Set the active widget in the details bar.
    SetDetailsBarWidget(DetailsBarWidget),

    /// Clear the details bar.
    ClearDetailsBar,

    Select(ResourceId),
    Unselect(ResourceId),
    ClearSelected,
}

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    /// Id of the `Project` the canvas is for.
    pub project: ResourceId,
    pub details_bar_widget: Option<DetailsBarWidget>,
    pub selected: HashSet<ResourceId>,
    show_side_bars: UseStateHandle<bool>,
}

impl CanvasState {
    pub fn new(project: ResourceId, show_side_bars: UseStateHandle<bool>) -> Self {
        Self {
            project,
            details_bar_widget: None,
            selected: HashSet::new(),
            show_side_bars,
        }
    }

    fn details_bar_widget_from_selected(&self) -> Option<DetailsBarWidget> {
        match self.selected.len() {
            0 => None,
            1 => {
                let Some(rid) = self.selected.iter().next() else {
                    return None;
                };

                Some(DetailsBarWidget::ContainerEditor(rid.clone()))
                // @todo: Editors for other resources.
                // DetailsBarWidget::AssetEditor(CoreAsset, Callback<CoreAsset>),
                // DetailsBarWidget::ScriptsAssociationsEditor(ResourceId, Option<Callback<()>>),
            }
            _ => {
                // @todo: Bulk editing.
                None
            }
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

            CanvasStateAction::Select(rid) => {
                current.selected.insert(rid);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::Unselect(rid) => {
                current.selected.remove(&rid);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::ClearSelected => {
                current.selected.clear();
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }
        }

        current.into()
    }
}

pub type CanvasStateReducer = UseReducerHandle<CanvasState>;

#[cfg(test)]
#[path = "./canvas_state_test.rs"]
mod canvas_state_test;
