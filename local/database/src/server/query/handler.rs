use crate::{query, Database};
use serde_json::Value as JsValue;

impl Database {
    pub fn handle_query_user(&self, query: query::User) -> JsValue {
        JsValue::Null
    }
}

impl Database {
    pub fn handle_query_project(&self, query: query::Project) -> JsValue {
        JsValue::Null
    }
}
