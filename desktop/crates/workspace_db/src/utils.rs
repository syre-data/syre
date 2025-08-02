use leptos::prelude::*;
use crate::types;
use syre_core::types::ResourceId;
use syre_desktop_ui_lib as ui_lib;

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
