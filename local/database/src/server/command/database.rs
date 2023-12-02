//! Handle [`DatabaseCommand`]s.
use super::super::Database;
use crate::command::DatabaseCommand;
use crate::constants::DATABASE_ID;
use serde_json::Value as JsValue;

impl Database {
    pub fn handle_command_database(&mut self, cmd: DatabaseCommand) -> JsValue {
        match cmd {
            DatabaseCommand::Kill => JsValue::Null,
            DatabaseCommand::Id => JsValue::String(DATABASE_ID.to_string()),
        }
    }
}
