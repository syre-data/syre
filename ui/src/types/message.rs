//! Message displayed to the user.
use uuid::Uuid;

/// Message types.
#[derive(Clone, PartialEq, Debug)]
pub enum MessageType {
    /// Informational message.
    Info,

    /// Success message.
    Success,

    /// Error message.
    Error,

    /// Warning message.
    Warning,
}

/// A Message.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    id: Uuid,

    /// Message to display.
    pub message: String,

    /// Expandable details.
    pub details: Option<String>,

    /// Type of message.
    pub kind: MessageType,
}

impl Message {
    /// Create a `Message` with a `kind` of [`MessageType::Info`].
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            details: None,
            kind: MessageType::Info,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Success`].
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            details: None,
            kind: MessageType::Success,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Error`].
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            details: None,
            kind: MessageType::Error,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Warning`].
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            details: None,
            kind: MessageType::Warning,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn set_details(&mut self, details: impl Into<String>) {
        self.details = Some(details.into());
    }

    pub fn clear_details(&mut self) {
        self.details = None;
    }
}
