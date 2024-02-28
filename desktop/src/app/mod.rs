//! Main application.
pub mod app;
pub mod app_state;
pub mod auth_state;
pub mod page_overlay;
pub mod projects_state;
pub mod shadow_box;

// Re-exports
pub use app::App;
pub use app_state::{AppStateAction, AppStateDispatcher, AppStateReducer, AppWidget};
pub use auth_state::{AuthStateAction, AuthStateReducer};
pub use page_overlay::PageOverlay;
pub use projects_state::{ProjectsStateAction, ProjectsStateDispatcher, ProjectsStateReducer};
pub use shadow_box::ShadowBox;
