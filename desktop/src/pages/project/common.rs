use super::state::workspace_graph::SelectedResource;
use leptos::ev::MouseEvent;
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
