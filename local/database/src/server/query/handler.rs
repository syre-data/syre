use crate::{constants, query, Database};
use serde_json::Value as JsValue;

impl Database {
    pub fn handle_query_config(&self, query: query::Config) -> JsValue {
        match query {
            query::Config::Id => constants::DATABASE_ID.into(),
        }
    }
}

impl Database {
    pub fn handle_query_state(&self, query: query::State) -> JsValue {
        match query {
            query::State::UserManifest => {
                serde_json::to_value(self.state.app().user_manifest()).unwrap()
            }
            query::State::ProjectManifest => {
                serde_json::to_value(self.state.app().project_manifest()).unwrap()
            }
        }
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
