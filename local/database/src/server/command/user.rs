//! User command handler.
use super::super::Database;
use crate::command::UserCommand;
use serde_json::Value as JsValue;
use thot_local::system::user_manifest::get_active_user;

impl Database {
    pub fn handle_command_user(&mut self, cmd: UserCommand) -> JsValue {
        match cmd {
            UserCommand::GetActive => {
                let user = get_active_user();
                serde_json::to_value(user).expect("could not convert `User` to JsValue")
            }
        }
    }
}
