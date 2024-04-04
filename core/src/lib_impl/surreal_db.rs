//! impls for Surreal DB.
use crate::types::ResourceId;
use surrealdb::sql::Id;

impl ResourceId {
    pub fn into_surreal_id(self) -> Id {
        self.into()
    }
}

impl Into<Id> for ResourceId {
    fn into(self) -> Id {
        Id::String(self.to_string())
    }
}
