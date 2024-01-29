//! State for th eproject workspace;
use super::details_bar::DetailsBarWidget;
use std::collections::HashSet;
use std::rc::Rc;
use syre_core::types::{ResourceId, ResourceMap};
use syre_ui::types::ContainerPreview;
use yew::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum ResourceType {
    Container,
    Asset,
}

#[derive(Debug)]
pub enum CanvasStateAction {
    /// Set the preview state.
    SetPreview(ContainerPreview),

    /// Set the active widget in the details bar.
    SetDetailsBarWidget(DetailsBarWidget),

    /// Clear the details bar.
    ClearDetailsBar,

    /// Mark a `Container` as selected.
    SelectContainer(ResourceId),

    /// Mark an `Asset` as selected.
    SelectAsset(ResourceId),

    /// Set `Asset` as only selected.
    SelectAssetOnly(ResourceId),

    /// Mark a resource as unselected.
    /// Updates the details bar as needed.
    Unselect(ResourceId),

    /// Mark multilpe resources unselected.
    /// Updates the details bar as needed.
    UnselectMany(Vec<ResourceId>),

    /// Clear selection state.
    ClearSelected,

    /// Set the visibility state of a `Container`.
    SetVisibility(ResourceId, bool),

    /// Removes a resource's mappings.
    Remove(ResourceId),

    /// Removes resource mappings.
    RemoveMany(Vec<ResourceId>),

    /// Toggle canvas drawers visibility.
    ToggleDrawers,

    /// Adds a flag to a resource.
    AddFlag {
        resource: ResourceId,
        message: String,
    },

    /// Removes the flag at the given index for the resource.
    RemoveFlag { resource: ResourceId, index: usize },

    /// Clears all the flags for the given resource.
    ClearResourceFlags(ResourceId),

    /// Clears all flags.
    ClearFlags,
}

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    /// Id of the `Project` the canvas is for.
    pub project: ResourceId,

    /// Active details bar widget.
    pub details_bar_widget: Option<DetailsBarWidget>,

    /// Selected resources.
    pub selected: HashSet<ResourceId>,

    /// Current preview view.
    pub preview: ContainerPreview,

    /// If canvas drawers are visible.
    pub drawers_visible: bool,

    /// Flag messages for each resource.
    pub flags: ResourceMap<Vec<String>>,

    /// `Container` tree visibility state.
    /// Key indicates the root of the hidden tree.
    visible: ResourceMap<bool>,

    /// Map of [`ResourceId`] to the type of the resource.
    resource_types: ResourceMap<ResourceType>,
    show_side_bars: UseStateHandle<bool>,
}

impl CanvasState {
    pub fn new(project: ResourceId, show_side_bars: UseStateHandle<bool>) -> Self {
        Self {
            preview: ContainerPreview::Assets,
            project,
            details_bar_widget: None,
            selected: HashSet::default(),
            drawers_visible: true,
            visible: ResourceMap::default(),
            flags: ResourceMap::default(),
            resource_types: ResourceMap::default(),
            show_side_bars,
        }
    }

    /// Returns the visibility state for a resource.
    pub fn is_visible(&self, rid: &ResourceId) -> bool {
        self.visible.get(&rid).unwrap_or(&true).to_owned()
    }

    /// Returns the ResourceType of a given ResourceId
    pub fn resource_type(&self, rid: &ResourceId) -> Option<ResourceType> {
        self.resource_types.get(rid).cloned()
    }

    /// Remove a resource from mappings.
    pub fn remove(&mut self, rid: &ResourceId) {
        self.selected.remove(rid);
        self.visible.remove(rid);
        self.resource_types.remove(rid);
    }

    #[tracing::instrument(skip(self))]
    fn details_bar_widget_from_selected(&self) -> Option<DetailsBarWidget> {
        match self.selected.len() {
            0 => None,
            1 => {
                let rid = self.selected.iter().next().expect("resource not available");
                let kind = self
                    .resource_types
                    .get(rid)
                    .expect("could not find resource type");

                match kind {
                    ResourceType::Container => Some(DetailsBarWidget::ContainerEditor(rid.clone())),
                    ResourceType::Asset => Some(DetailsBarWidget::AssetEditor(rid.clone())),
                }
            }
            _ => {
                let mut kinds = self.selected.iter().map(|rid| {
                    self.resource_types
                        .get(rid)
                        .expect("could not find resource type")
                });

                // must clone iterator, iterators can only be used once
                if kinds.clone().all(|k| k == &ResourceType::Container) {
                    Some(DetailsBarWidget::ContainerBulkEditor(self.selected.clone()))
                } else if kinds.all(|k| k == &ResourceType::Asset) {
                    Some(DetailsBarWidget::AssetBulkEditor(self.selected.clone()))
                } else {
                    Some(DetailsBarWidget::MixedBulkEditor(self.selected.clone()))
                }
            }
        }
    }
}

impl Reducible for CanvasState {
    type Action = CanvasStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        tracing::debug!(?action);
        let mut current = (*self).clone();
        match action {
            CanvasStateAction::SetPreview(preview) => {
                current.preview = preview;
            }

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

            CanvasStateAction::SelectAssetOnly(rid) => {
                current.selected.clear();
                current.selected.insert(rid.clone());
                current.resource_types.insert(rid, ResourceType::Asset);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::Unselect(rid) => {
                current.selected.remove(&rid);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::UnselectMany(rids) => {
                for rid in rids {
                    current.selected.remove(&rid);
                }
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::ClearSelected => {
                current.selected.clear();
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::SetVisibility(rid, visible) => {
                current.visible.insert(rid, visible);
            }

            CanvasStateAction::Remove(rid) => {
                current.remove(&rid);
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::RemoveMany(rids) => {
                for rid in rids {
                    current.remove(&rid);
                }
                current.details_bar_widget = current.details_bar_widget_from_selected();
            }

            CanvasStateAction::ToggleDrawers => {
                current.drawers_visible = !current.drawers_visible;
            }

            CanvasStateAction::AddFlag { resource, message } => {
                let resource_flags = current.flags.entry(resource).or_insert(Vec::new());
                resource_flags.push(message);
            }

            CanvasStateAction::RemoveFlag { resource, index } => {
                if let Some(resource_flags) = current.flags.get_mut(&resource) {
                    resource_flags.remove(index);
                }
            }

            CanvasStateAction::ClearResourceFlags(resource) => {
                current.flags.remove(&resource);
            }

            CanvasStateAction::ClearFlags => {
                current.flags.clear();
            }
        }

        current.into()
    }
}

pub type CanvasStateReducer = UseReducerHandle<CanvasState>;
pub type CanvasStateDispatcher = UseReducerDispatcher<CanvasState>;
