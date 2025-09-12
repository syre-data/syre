use crate::types;
use leptos::prelude::*;
use syre_core::types::ResourceId;
use syre_desktop_ui_lib as ui_lib;

pub fn asset_title_closure(asset: &ui_lib::state::Asset) -> impl Fn() -> String + use<> {
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
            if path.is_empty() { None } else { Some(path) }
        }) {
            path
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!("invalid asset: no name or path");
            "(invalid asset)".to_string()
        }
    }
}

/// # Arguments
/// + `select_multiple`: Should multiple resources be selected.
/// Usually indicated by the `shift` key being held.
pub fn interpret_resource_selection_action(
    rid: &ResourceId,
    selected_resources: &Vec<ui_lib::state::workspace_graph::Resource>,
    select_multiple: bool,
) -> types::SelectionAction {
    if select_multiple {
        if selected_resources
            .iter()
            .find(|resource| resource.rid().with_untracked(|resource| resource == rid))
            .is_some()
        {
            types::SelectionAction::Unselect
        } else {
            types::SelectionAction::Select
        }
    } else {
        let is_only_selected = if let [resource] = &selected_resources[..] {
            resource
                .rid()
                .with_untracked(|selected_id| rid == selected_id)
        } else {
            false
        };

        if is_only_selected {
            types::SelectionAction::Clear
        } else {
            types::SelectionAction::SelectOnly
        }
    }
}
