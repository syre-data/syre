use crate::{query, Database};
use serde_json::Value as JsValue;

impl Database {
    pub fn handle_query_config(&self, query: query::Config) -> JsValue {
        todo!();
    }
}

impl Database {
    pub fn handle_query_user(&self, query: query::User) -> JsValue {
        todo!();
    }
}

impl Database {
    pub fn handle_query_project(&self, query: query::Project) -> JsValue {
        todo!();
    }
}
