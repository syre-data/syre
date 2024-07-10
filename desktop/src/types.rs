use leptos::*;

/// Enum for different mouse buttons
/// for use with `MouseEvent::button`.
/// See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button#value.
pub enum MouseButton {
    Primary = 0,
    Auxillary = 1,
    Secondary = 2,
    Fourth = 3,
    Fifth = 4,
}

/// App wide messages.
#[derive(Clone, derive_more::Deref)]
pub struct Messages(RwSignal<Vec<crate::components::Message>>);
impl Messages {
    pub fn new() -> Self {
        Self(create_rw_signal(vec![]))
    }
}
