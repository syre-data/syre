pub(crate) mod filter;
mod project_bar;
mod state;
mod utils;
mod workspace;

pub use workspace::Workspace;

mod types {
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
}
