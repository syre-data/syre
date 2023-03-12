//! Hooks related to the canvas state.
pub mod details_bar_widget;
pub mod graph;
pub mod project;

// Re-export
pub use details_bar_widget::use_details_bar_widget;
pub use graph::use_project_graph;
pub use project::use_canvas_project;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
