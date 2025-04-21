pub enum SelectionAction {
    /// resource should be removed from the selection.
    Unselect,

    /// Resource should be added to the selection.
    Select,

    /// Resource should be the only selected.
    SelectOnly,

    /// Selection should be cleared.
    Clear,
}
