//! Handle analysis commands.
use super::super::Database;
use crate::command::RunnerCommand;
use crate::event::{Analysis as AnalysisUpdate, Update};
use serde_json::Value as JsValue;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_runner(&mut self, cmd: RunnerCommand) -> JsValue {
        match cmd {
            RunnerCommand::Flag { resource, message } => {
                let project = if let Some(project) = self.store.get_container_project(&resource) {
                    project.clone()
                } else if let Some(container) = self.store.get_asset_container_id(&resource) {
                    self.store.get_container_project(container).unwrap().clone()
                } else {
                    tracing::debug!("resource `{resource:?}` not found");
                    panic!("resource not found");
                };

                self.publish_update(&Update::Project {
                    project,
                    update: AnalysisUpdate::Flag { resource, message }.into(),
                })
                .unwrap();

                serde_json::to_value(JsValue::Null).unwrap()
            }
        }
    }
}
