//! Main application.
pub mod app;
pub mod app_state;
pub mod auth_state;
pub mod projects_state;
pub mod shadow_box;

// Re-exports
pub use app::App;
pub use app_state::{AppStateAction, AppStateReducer, AppWidget};
pub use auth_state::{AuthStateAction, AuthStateReducer};
pub use projects_state::{ProjectsStateAction, ProjectsStateReducer};
pub use shadow_box::ShadowBox;
