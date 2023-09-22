//! Database for storing resources.
#[path = "./handler/mod.rs"]
mod handler;

use super::store::Datastore;
use crate::command::Command;
use crate::constants::{DATABASE_ID, REQ_REP_PORT};
use crate::{Error, Result};
use notify::{self, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use serde_json::Value as JsValue;
use std::net::Ipv4Addr;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;

static LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
static FILE_SYSTEM_EVENT_BUFFER_SIZE: usize = 1;

/// Database.
pub struct Database {
    store: Datastore,

    /// Kill flag indicating the database should stop listening and return.
    kill: bool,

    /// ZMQ context.
    zmq_context: zmq::Context,

    // File watcher
    watcher: Debouncer<RecommendedWatcher, FileIdMap>,

    // File system event receiver
    fs_rx: mpsc::Receiver<DebounceEventResult>,
}

impl Database {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(FILE_SYSTEM_EVENT_BUFFER_SIZE);
        let watcher = new_debouncer(
            Duration::from_millis(1000),
            None,
            move |event: DebounceEventResult| {
                let tx = tx.clone();
                futures::executor::block_on(async {
                    tx.send(event).await.unwrap();
                });
            },
        )
        .unwrap();

        Database {
            store: Datastore::new(),
            kill: false,
            zmq_context: zmq::Context::new(),
            watcher,
            fs_rx: rx,
        }
    }

    /// Start the database.
    pub async fn start(&mut self) {
        tokio::spawn(self.listen_for_commands());
        tokio::spawn(self.listen_for_file_system_events());
    }

    fn watch(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.watcher
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .unwrap();

        self.watcher
            .cache()
            .add_root(path, RecursiveMode::Recursive);
    }

    #[tracing::instrument(skip(self))]
    async fn listen_for_commands(&mut self) -> Result {
        let rep_socket = self.zmq_context.socket(zmq::SocketType::REP)?;
        rep_socket.bind(&format!("tcp://{LOCALHOST}:{REQ_REP_PORT}"))?;

        loop {
            if self.kill {
                break;
            }

            let cmd = self.receive_command(&rep_socket).await?;
            tracing::debug!(?cmd);

            let res = self.handle_command(cmd);
            rep_socket
                .send(&res.to_string(), 0)
                .expect("could not send response");
        }

        Ok(())
    }

    async fn receive_command(&self, socket: &zmq::Socket) -> Result<Command> {
        let mut msg = zmq::Message::new();
        socket.recv(&mut msg, 0).expect("could not recieve request");

        let Some(msg_str) = msg.as_str() else {
            let err_msg = "invalid message: could not convert to string";
            tracing::debug!(?err_msg);
            return Err(Error::ZMQ(err_msg.into()));
        };

        let Ok(cmd) = serde_json::from_str(msg_str) else {
            let err_msg = "invalid message: could not convert `Message` to `Command";
            tracing::debug!(err = err_msg, msg = msg_str);
            return Err(Error::ZMQ(err_msg.into()));
        };

        Ok(cmd)
    }

    // TODO Handle errors.
    /// Handles a given command, returning the correct data.
    fn handle_command(&mut self, command: Command) -> JsValue {
        match command {
            Command::AssetCommand(cmd) => self.handle_command_asset(cmd),
            Command::ContainerCommand(cmd) => self.handle_command_container(cmd),
            Command::DatabaseCommand(cmd) => self.handle_command_database(cmd),
            Command::ProjectCommand(cmd) => self.handle_command_project(cmd),
            Command::GraphCommand(cmd) => self.handle_command_graph(cmd),
            Command::ScriptCommand(cmd) => self.handle_command_script(cmd),
            Command::UserCommand(cmd) => self.handle_command_user(cmd),
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
