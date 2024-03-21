//! Analysis types.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syre_core::project::{ExcelTemplate, Script};
use syre_core::types::ResourceId;

pub type Store = HashMap<ResourceId, AnalysisKind>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum AnalysisKind {
    Script(Script),
    ExcelTemplate(ExcelTemplate),
}

impl From<Script> for AnalysisKind {
    fn from(value: Script) -> Self {
        Self::Script(value)
    }
}

impl From<ExcelTemplate> for AnalysisKind {
    fn from(value: ExcelTemplate) -> Self {
        Self::ExcelTemplate(value)
    }
}
