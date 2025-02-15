use crate::Command;
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
}
