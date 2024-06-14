//! Query.
use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use syre_core::types::ResourceId;

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchCommand {
    Search(String),
    Query(Query),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub select: Field,
    pub limit: Option<usize>,
    pub project: Option<ResourceId>,
    pub resource_kind: Option<ResourceKind>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Field {
    Name(Option<String>),
    Kind(Option<String>),
    Tag(String),
    Description(String),
    Metadata(Metadatum),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadatum {
    key: String,
    value: JsValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ResourceKind {
    Container,
    Asset,
}

pub struct QueryBuilder {
    select: Field,
    limit: Option<usize>,
    project: Option<ResourceId>,
    resource_kind: Option<ResourceKind>,
}

impl QueryBuilder {
    pub fn new(select: Field) -> Self {
        Self {
            select,
            limit: None,
            project: None,
            resource_kind: None,
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    pub fn clear_limit(&mut self) {
        self.limit = None;
    }

    pub fn set_project(&mut self, project: ResourceId) {
        self.project = Some(project);
    }

    pub fn clear_project(&mut self) {
        self.project = None;
    }

    pub fn set_resource_kind(&mut self, resource_kind: ResourceKind) {
        self.resource_kind = Some(resource_kind);
    }

    pub fn clear_resource_kind(&mut self) {
        self.resource_kind = None;
    }

    pub fn build(self) -> Query {
        let Self {
            select,
            limit,
            project,
            resource_kind,
        } = self;

        Query {
            select,
            limit,
            project,
            resource_kind,
        }
    }
}
