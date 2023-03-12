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
}

/// A Message.
#[derive(Clone, PartialEq, Debug)]
pub struct Message<'a> {
    id: Uuid,

    /// Message to display.
    pub message: &'a str,

    /// Type of message.
    pub kind: MessageType,
}

impl<'a> Message<'a> {
    /// Create a `Message` with a `kind` of [`MessageType::Info`].
    pub fn info(message: &'a str) -> Self {
        Self {
            id: Uuid::new_v4(),
            message,
            kind: MessageType::Info,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Success`].
    pub fn success(message: &'a str) -> Self {
        Self {
            id: Uuid::new_v4(),
            message,
            kind: MessageType::Success,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Error`].
    pub fn error(message: &'a str) -> Self {
        Self {
            id: Uuid::new_v4(),
            message,
            kind: MessageType::Error,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

#[cfg(test)]
#[path = "./message_test.rs"]
mod message_test;
