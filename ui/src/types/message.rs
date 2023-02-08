//! Message displayed to the user.

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
pub struct Message {
    /// Message to display.
    pub message: String,

    /// Type of message.
    pub kind: MessageType,
}

impl Message {
    /// Create a `Message` with a `kind` of [`MessageType::Info`].
    pub fn info(message: String) -> Self {
        Self {
            message,
            kind: MessageType::Info,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Success`].
    pub fn success(message: String) -> Self {
        Self {
            message,
            kind: MessageType::Success,
        }
    }

    /// Create a `Message` with a `kind` of [`MessageType::Error`].
    pub fn error(message: String) -> Self {
        Self {
            message,
            kind: MessageType::Error,
        }
    }
}

#[cfg(test)]
#[path = "./message_test.rs"]
mod message_test;
