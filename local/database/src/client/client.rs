//! Client to interact with a [`Database`].
use crate::{common, constants::LOCALHOST, types::PortNumber, Query};
use serde_json::Value as JsValue;
use std::net::TcpListener;

static CONNECT_TIMEOUT: i32 = 5000;
static RECV_TIMEOUT: i32 = 5000;

pub type CmdResult<T, E> = zmq::Result<Result<T, E>>;

/// Checks if a given port on the loopback address is free.
fn port_is_free(port: PortNumber) -> bool {
    TcpListener::bind(format!("{LOCALHOST}:{port}")).is_ok()
}

pub struct Client {
    config: Config,
    zmq_context: zmq::Context,
    state: state::Client,
}

impl Client {
    pub fn new() -> Self {
        let config = Config::new(CONNECT_TIMEOUT, RECV_TIMEOUT);
        let ctx = zmq::Context::new();
        Self {
            config: config.clone(),
            zmq_context: ctx.clone(),
            state: state::Client::new(config.clone(), ctx.clone()),
        }
    }

    pub fn send(&self, query: Query) -> zmq::Result<JsValue> {
        // TODO: May be able to move creation of `socket` to `#new`, but may run into `Sync` issues.
        let socket = self.socket();
        socket.send(&serde_json::to_string(&query).unwrap(), 0)?;

        let mut msg = zmq::Message::new();
        socket.recv(&mut msg, 0)?;

        Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
    }

    pub fn state(&self) -> &state::Client {
        &self.state
    }

    fn socket(&self) -> zmq::Socket {
        const SOCKET_KIND: zmq::SocketType = zmq::REQ;
        let socket = self.zmq_context.socket(SOCKET_KIND).unwrap();
        socket
            .set_connect_timeout(self.config.connect_timeout())
            .unwrap();
        socket.set_rcvtimeo(self.config.recv_timeout()).unwrap();
        socket
            .connect(&common::zmq_url(SOCKET_KIND).unwrap())
            .unwrap();

        socket
    }
}

mod state {
    use super::Config;
    use crate::{
        common, query,
        state::{self, ManifestState},
        Query,
    };
    use serde_json::Value as JsValue;
    use std::path::PathBuf;
    use syre_core::system::User;

    pub struct Client {
        config: Config,
        zmq_context: zmq::Context,
    }

    impl Client {
        pub(super) fn new(config: Config, zmq_context: zmq::Context) -> Self {
            Self {
                config,
                zmq_context,
            }
        }

        pub fn user_manifest(&self) -> zmq::Result<ManifestState<User>> {
            let state = self.send(query::State::ProjectManifest.into())?;
            Ok(serde_json::from_value(state).unwrap())
        }

        pub fn project_manifest(&self) -> zmq::Result<ManifestState<PathBuf>> {
            let state = self.send(query::State::ProjectManifest.into())?;
            Ok(serde_json::from_value(state).unwrap())
        }

        pub fn projects(&self) -> zmq::Result<Vec<crate::state::Project>> {
            let state = self.send(query::State::Projects.into())?;
            Ok(serde_json::from_value(state).unwrap())
        }

        /// Retrieve the entrie graph of a project.
        ///
        /// # Arguments
        /// 1. `project`: Base path of the project.
        ///
        /// # Returns
        /// `None` if the project or graph does not exist.
        pub fn graph(&self, project: impl Into<PathBuf>) -> zmq::Result<Option<state::Graph>> {
            let state = self.send(query::State::Graph(project.into()).into())?;
            Ok(serde_json::from_value(state).unwrap())
        }

        /// Retrieve the state of a container.
        ///
        /// # Arguments
        /// 1. `project`: Base path of the project.
        /// 2. `container`: Relative path to the container from the data root.
        ///
        /// # Returns
        /// `None` if the project or container does not exist.
        pub fn container(
            &self,
            project: impl Into<PathBuf>,
            container: impl Into<PathBuf>,
        ) -> zmq::Result<Option<state::Container>> {
            let state = self.send(
                query::State::Container {
                    project: project.into(),
                    container: container.into(),
                }
                .into(),
            )?;

            Ok(serde_json::from_value(state).unwrap())
        }

        /// Retrieve the state of a container.
        ///
        /// # Arguments
        /// 1. `project`: Base path of the project.
        /// 2. `container`: Relative path to the container from the data root.
        /// 3. `asset`: Relative path to the asset from the container.
        ///
        /// # Returns
        /// `None` if the project, container, or asset does not exist.
        pub fn asset(
            &self,
            project: impl Into<PathBuf>,
            container: impl Into<PathBuf>,
            asset: impl Into<PathBuf>,
        ) -> zmq::Result<Option<state::Asset>> {
            let state = self.send(
                query::State::Asset {
                    project: project.into(),
                    container: container.into(),
                    asset: asset.into(),
                }
                .into(),
            )?;

            Ok(serde_json::from_value(state).unwrap())
        }

        fn send(&self, query: Query) -> zmq::Result<JsValue> {
            // TODO: May be able to move creation of `socket` to `#new`, but may run into `Sync` issues.
            let socket = self.socket();
            socket.send(&serde_json::to_string(&query).unwrap(), 0)?;

            let mut msg = zmq::Message::new();
            socket.recv(&mut msg, 0)?;

            Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
        }

        fn socket(&self) -> zmq::Socket {
            const SOCKET_KIND: zmq::SocketType = zmq::REQ;
            let socket = self.zmq_context.socket(SOCKET_KIND).unwrap();
            socket
                .set_connect_timeout(self.config.connect_timeout())
                .unwrap();
            socket.set_rcvtimeo(self.config.recv_timeout()).unwrap();
            socket
                .connect(&common::zmq_url(SOCKET_KIND).unwrap())
                .unwrap();

            socket
        }
    }
}

#[derive(Clone)]
struct Config {
    connect_timeout: i32,
    recv_timeout: i32,
}

impl Config {
    pub fn new(connect_timeout: i32, recv_timeout: i32) -> Self {
        Self {
            connect_timeout,
            recv_timeout,
        }
    }

    pub fn connect_timeout(&self) -> i32 {
        self.connect_timeout
    }

    pub fn recv_timeout(&self) -> i32 {
        self.recv_timeout
    }
}
