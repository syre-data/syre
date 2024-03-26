//! Handle query commands.
use super::super::Database;
use crate::command::SearchCommand;
use serde_json::Value as JsValue;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_search(&self, command: SearchCommand) -> JsValue {
        match command {
            SearchCommand::Search(query) => {
                let res = self.search.search(&self.store, query);
                serde_json::to_value(&res).unwrap()
            }

            SearchCommand::Query(query) => {
                let res = self.search.query(&self.store, query);
                serde_json::to_value(&res).unwrap()
            }
        }
    }
}
