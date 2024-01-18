//! Project canvas.
pub mod asset;
pub mod canvas;
pub mod canvas_state;
pub mod container;
pub mod details_bar;
pub mod graph_state;
pub mod layers_bar;
pub mod project;
pub mod project_controls;
pub mod selection_action;

// Re-exports
pub use canvas::ProjectCanvas;
pub use canvas_state::{CanvasStateAction, CanvasStateDispatcher, CanvasStateReducer};
pub use graph_state::{GraphStateAction, GraphStateDispatcher, GraphStateReducer};
pub use project_controls::ProjectControls;
