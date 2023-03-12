//! Project canvas.
pub mod canvas;
pub mod canvas_state;
pub mod graph_state;

// Re-exports
pub use canvas::ProjectCanvas;
pub use canvas_state::{CanvasStateAction, CanvasStateReducer};
pub use graph_state::{GraphStateAction, GraphStateReducer};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
