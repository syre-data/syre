use syre_core::project::{ExcelTemplate, Script};
use syre_local::types::AnalysisKind;

pub trait DisplayName {
    fn display_name(&self) -> String;
}

impl DisplayName for Script {
    fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }
}

impl DisplayName for ExcelTemplate {
    fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.template.path.to_string_lossy().to_string())
    }
}

impl DisplayName for AnalysisKind {
    fn display_name(&self) -> String {
        match self {
            AnalysisKind::Script(script) => script.display_name(),
            AnalysisKind::ExcelTemplate(template) => template.display_name(),
        }
    }
}
