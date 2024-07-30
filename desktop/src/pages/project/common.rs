use super::state::workspace_graph::SelectedResource;
use crate::pages::project::state;
use leptos::{ev::MouseEvent, *};
use syre_core::types::ResourceId;

pub fn interpret_resource_selection_action(
    resource: &ResourceId,
    event: &MouseEvent,
    selection: &Vec<SelectedResource>,
) -> SelectionAction {
    if event.shift_key() {
        let is_selected = selection
            .iter()
            .any(|s_resource| s_resource.rid() == resource);

        if is_selected {
            SelectionAction::Remove
        } else {
            SelectionAction::Add
        }
    } else {
        let is_only_selected = if let [s_resource] = &selection[..] {
            s_resource.rid() == resource
        } else {
            false
        };

        if is_only_selected {
            SelectionAction::Clear
        } else {
            SelectionAction::SelectOnly
        }
    }
}

pub enum SelectionAction {
    /// resource should be removed from the selection.
    Remove,

    /// Resource should be added to the selection.
    Add,

    /// Resource should be the only selected.
    SelectOnly,

    /// Selection should be cleared.
    Clear,
}

pub fn asset_title_closure(asset: &state::Asset) -> impl Fn() -> String {
    let name = asset.name();
    let path = asset.path();
    move || {
        if let Some(name) = name.with(|name| {
            if let Some(name) = name {
                if name.is_empty() {
                    None
                } else {
                    Some(name.clone())
                }
            } else {
                None
            }
        }) {
            name
        } else if let Some(path) = path.with(|path| {
            let path = path.to_string_lossy().trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        }) {
            path
        } else {
            tracing::error!("invalid asset: no name or path");
            "(invalid asset)".to_string()
        }
    }
}
