//! Hooks related to the canvas state.
pub mod details_bar_widget;
pub mod load_project_graph;
pub mod project;

// Re-export
pub use details_bar_widget::use_details_bar_widget;
pub use load_project_graph::use_load_project_graph;
pub use project::use_canvas_project;
