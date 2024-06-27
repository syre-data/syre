//! Selection action based on click event.
use yew::prelude::MouseEvent;

pub enum SelectionAction {
    SelectOnly,
    Select,
    Unselect,
}

/// Determines the selection action from the current action and state.
///
/// # Arguments
/// 1. `selected`: If the clicked resource is currently selected.
/// 2. `multiple`: If at least one other resource is currently selected.
/// 3. `e`: The [`MouseEvent`].
pub fn selection_action(selected: bool, multiple: bool, e: MouseEvent) -> SelectionAction {
    if e.shift_key() {
        if selected {
            return SelectionAction::Unselect;
        } else {
            return SelectionAction::Select;
        }
    }

    if selected {
        if multiple {
            return SelectionAction::SelectOnly;
        }

        return SelectionAction::Unselect;
    }

    SelectionAction::SelectOnly
}
