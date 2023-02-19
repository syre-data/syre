//! State for th eproject workspace;
use crate::components::details_bar::DetailsBarWidget;
use std::collections::HashSet;
use std::rc::Rc;
use thot_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;

#[derive(PartialEq, Clone)]
enum ResourceType {
    Container,
    Asset,
}

pub enum CanvasStateAction {
    /// Set the active widget in the details bar.
    SetDetailsBarWidget(DetailsBarWidget),

    /// Clear the details bar.
    ClearDetailsBar,

    /// Mark a `Container` as selected.
    SelectContainer(ResourceId),

    /// Mark an `Asset` as selected.
    SelectAsset(ResourceId),

    /// Mark a resource as unselected.
    Unselect(ResourceId),

    /// Clear selection state.
    ClearSelected,

    /// Set the visibility state of a `Container`.
    SetVisibility(ResourceId, bool),
}

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    /// Id of the `Project` the canvas is for.
    pub project: ResourceId,

    /// Active details bar widget.
    pub details_bar_widget: Option<DetailsBarWidget>,

    /// Selected resources.
    pub selected: HashSet<ResourceId>,

    /// `Container` tree visibility state.
    /// Key indicates the tree root, whose children are affected.
    visible: ResourceMap<bool>,

    /// Map of [`ResourceId`] to the type of the resource.
    resource_types: ResourceMap<ResourceType>,
    show_side_bars: UseStateHandle<bool>,
}

impl CanvasState {
    pub fn new(project: ResourceId, show_side_bars: UseStateHandle<bool>) -> Self {
        Self {
            project,
            details_bar_widget: None,
            selected: HashSet::default(),
            visible: ResourceMap::default(),
            resource_types: ResourceMap::default(),
            show_side_bars,
        }
    }

    /// Returns the visibility state for a resource.
    pub fn is_visible(&self, rid: &ResourceId) -> bool {
        self.visible.get(&rid).unwrap_or(&true).to_owned()
    }

    fn details_bar_widget_from_selected(&self) -> Option<DetailsBarWidget> {
        match self.selected.len() {
            0 => None,
            1 => {
                let Some(rid) = self.selected.iter().next() else {
                    return None;
                };

                let kind = self
                    .resource_types
                    .get(&rid)
                    .expect("could not find resource type");

                match kind {
                    ResourceType::Container => Some(DetailsBarWidget::ContainerEditor(rid.clone())),
                    ResourceType::Asset => Some(DetailsBarWidget::AssetEditor(rid.clone())),
                }
                // @todo: Editors for other resources.
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

            CanvasStateAction::SelectContainer(rid) => {
                current.selected.insert(rid.clone());
                current.resource_types.insert(rid, ResourceType::Container);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::SelectAsset(rid) => {
                current.selected.insert(rid.clone());
                current.resource_types.insert(rid, ResourceType::Asset);
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

            CanvasStateAction::SetVisibility(rid, visible) => {
                current.visible.insert(rid, visible);
            }
        }

        current.into()
    }
}

pub type CanvasStateReducer = UseReducerHandle<CanvasState>;

#[cfg(test)]
#[path = "./canvas_state_test.rs"]
mod canvas_state_test;
