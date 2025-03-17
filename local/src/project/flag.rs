use serde::{Deserialize, Serialize};
use syre_core::types::ResourceId;

pub type Id = uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flag {
    id: Id,
    severity: Severity,
    message: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
}

impl Flag {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            id: Id::now_v7(),
            severity,
            message: message.into(),
            source: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(Severity::Info, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    pub fn set_source(&mut self, source: Source) {
        let _ = self.source.insert(source);
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn source(&self) -> &Option<Source> {
        &self.source
    }
}

/// Source of the flag.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Source {
    script: ResourceId,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
}

impl Source {
    pub fn script(&self) -> &ResourceId {
        &self.script
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Severity {
    Info,
    Warning,
    Error,
}
