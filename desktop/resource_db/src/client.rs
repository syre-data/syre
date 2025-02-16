use crate::{Command, SearchResult};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

pub struct Client {
    query_tx: mpsc::UnboundedSender<Command>,
}

impl Client {
    pub fn new(query_tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { query_tx }
    }

    pub fn query(&self, query: impl Into<String>) -> surrealdb::Result<surrealdb::Response> {
        let (tx, rx) = oneshot::channel();
        let cmd = Command::Query {
            tx,
            query: query.into(),
        };

        self.query_tx.send(cmd).unwrap();
        rx.blocking_recv().unwrap()
    }

    pub fn search(&self, query: impl Into<String>) -> surrealdb::Result<SearchResult> {
        let (tx, rx) = oneshot::channel();
        let cmd = Command::Search {
            tx,
            query: query.into(),
        };

        self.query_tx.send(cmd).unwrap();
        rx.blocking_recv().unwrap()
    }

    pub fn search_project(
        &self,
        query: impl Into<String>,
        project: PathBuf,
    ) -> surrealdb::Result<SearchResult> {
        let (tx, rx) = oneshot::channel();
        let cmd = Command::SearchProject {
            tx,
            query: query.into(),
            project,
        };

        self.query_tx.send(cmd).unwrap();
        rx.blocking_recv().unwrap()
    }
}
