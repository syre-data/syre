use leptos::prelude::*;

/// Enum for different mouse buttons
/// for use with `MouseEvent::button`.
/// See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button#value.
#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Primary = 0,
    // Auxillary = 1,
    // Secondary = 2,
    // Fourth = 3,
    // Fifth = 4,
}

impl PartialEq<i16> for MouseButton {
    fn eq(&self, other: &i16) -> bool {
        (*self as i16).eq(other)
    }
}

impl PartialEq<MouseButton> for i16 {
    fn eq(&self, other: &MouseButton) -> bool {
        other.eq(self)
    }
}

/// Data view for the main workspace.
#[derive(Clone, Copy, Default)]
pub enum DataView {
    #[default]
    Graph,
    Database,
}

// TODO: Should be a `Theme` enum.
/// User prefers dark theme.
#[derive(derive_more::Deref, Clone, Copy)]
pub struct PrefersDarkTheme(RwSignal<bool>);
impl PrefersDarkTheme {
    pub fn new(prefers_dark: bool) -> Self {
        Self(RwSignal::new(prefers_dark))
    }
}

pub mod container {
    use serde::{Deserialize, Serialize};
    use syre_core::types::ResourceId;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Action {
        /// Add an analysis association to the Container.
        AddAnalysisAssociation(ResourceId),
    }
}
