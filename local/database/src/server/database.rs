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

static LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Database.
pub struct Database {
    store: Datastore,

    /// Kill flag indicating the database should stop listening and return.
    kill: bool,

    /// ZMQ context.
    zmq_context: zmq::Context,

    // File watchers
    watcher: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl Database {
    pub fn new() -> Self {
        let watcher = new_debouncer(
            Duration::from_millis(1000),
            None,
            |event: DebounceEventResult| Self::handle_file_system_events(event),
        )
        .unwrap();

        Database {
            store: Datastore::new(),
            kill: false,
            zmq_context: zmq::Context::new(),
            watcher,
        }
    }

    pub fn watcher_mut(&mut self) -> &mut Debouncer<RecommendedWatcher, FileIdMap> {
        &mut self.watcher
    }

    pub fn watch(&mut self, path: impl AsRef<Path>) {
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
    pub fn listen_for_commands(&mut self) -> Result {
        let rep_socket = self.zmq_context.socket(zmq::SocketType::REP)?;
        rep_socket.bind(&format!("tcp://{LOCALHOST}:{REQ_REP_PORT}"))?;

        loop {
            if self.kill {
                break;
            }

            let mut msg = zmq::Message::new();
            rep_socket
                .recv(&mut msg, 0)
                .expect("could not recieve request");

            let Some(msg_str) = msg.as_str() else {
                let err_msg = "invalid message: could not convert to string".to_string();
                tracing::debug!(?err_msg);

                let res: Result<JsValue> = Err(Error::ZMQ(err_msg));
                let res = serde_json::to_value(res).expect("could not convert error to JsValue");
                rep_socket
                    .send(&res.to_string(), 0)
                    .expect("could not send response");

                continue;
            };

            let Ok(cmd) = serde_json::from_str(msg_str) else {
                let err_msg =
                    "invalid message: could not convert `Message` to `Command".to_string();
                tracing::debug!(err = err_msg, msg = msg_str);
                let res: Result<JsValue> = Err(Error::ZMQ(err_msg));
                let res = serde_json::to_value(res).expect("could not convert error to JsValue");
                rep_socket
                    .send(&res.to_string(), 0)
                    .expect("could not send response");

                continue;
            };

            tracing::debug!(?cmd);
            let res = self.handle_command(cmd);

            rep_socket
                .send(&res.to_string(), 0)
                .expect("could not send response");
        }

        Ok(())
    }

    // TODO Handle errors.
    /// Handles a given command, returning the correct data.
    pub fn handle_command(&mut self, command: Command) -> JsValue {
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
