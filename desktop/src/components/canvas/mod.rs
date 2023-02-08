//! Project canvas.
pub mod canvas;
pub mod canvas_state;
pub mod container_tree_state;
// pub mod navbar;

// Re-exports
pub use canvas::ProjectCanvas;
pub use canvas_state::{CanvasStateAction, CanvasStateReducer};
pub use container_tree_state::{ContainerTreeStateAction, ContainerTreeStateReducer};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
