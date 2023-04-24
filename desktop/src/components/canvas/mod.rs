//! Project canvas.
pub mod asset;
pub mod canvas;
pub mod canvas_state;
pub mod container;
pub mod details_bar;
pub mod graph_state;
pub mod project;

// Re-exports
pub use canvas::ProjectCanvas;
pub use canvas_state::{CanvasStateAction, CanvasStateReducer};
pub use graph_state::{GraphStateAction, GraphStateReducer};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
