//! Client to interact with a [`Database`].
use crate::command::{Command, DatabaseCommand};
use crate::common;
use crate::constants::{DATABASE_ID, LOCALHOST, REQ_REP_PORT};
use crate::types::PortNumber;
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
    zmq_context: zmq::Context,
    project: project::Client,
    graph: graph::Client,
    container: container::Client,
    asset: asset::Client,
}

impl Client {
    pub fn new() -> Self {
        let ctx = zmq::Context::new();
        Self {
            zmq_context: ctx.clone(),
            project: project::Client::new(ctx.clone()),
            graph: graph::Client::new(ctx.clone()),
            container: container::Client::new(ctx.clone()),
            asset: asset::Client::new(ctx.clone()),
        }
    }

    pub fn send(&self, cmd: Command) -> zmq::Result<JsValue> {
        // TODO: May be able to move creation of `socket` to `#new`, but may run into `Sync` issues.
        let socket = Self::socket(&self.zmq_context);
        socket.send(&serde_json::to_string(&cmd).unwrap(), 0)?;

        let mut msg = zmq::Message::new();
        socket.recv(&mut msg, 0)?;

        Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
    }

    pub fn project(&self) -> &project::Client {
        &self.project
    }

    pub fn graph(&self) -> &graph::Client {
        &self.graph
    }

    pub fn container(&self) -> &container::Client {
        &self.container
    }

    pub fn asset(&self) -> &asset::Client {
        &self.asset
    }

    /// Checks if a database is running.
    pub fn server_available() -> bool {
        if port_is_free(REQ_REP_PORT) {
            // port is open, no chance of a listener
            return false;
        }

        let ctx = zmq::Context::new();
        let socket = Self::socket(&ctx);
        socket
            .send(
                &serde_json::to_string(&Command::Database(DatabaseCommand::Id)).unwrap(),
                0,
            )
            .unwrap();

        let mut msg = zmq::Message::new();
        let res = socket.recv(&mut msg, 0);
        if res.is_err() {
            // TODO Check error type for timeout.
            return false;
        }

        let Some(id_str) = msg.as_str() else {
            panic!("invalid response");
        };

        let id_str: &str = serde_json::from_str(id_str).unwrap();

        return id_str == DATABASE_ID;
    }

    fn socket(ctx: &zmq::Context) -> zmq::Socket {
        const SOCKET_KIND: zmq::SocketType = zmq::REQ;
        let socket = ctx.socket(SOCKET_KIND).unwrap();
        socket.set_connect_timeout(CONNECT_TIMEOUT).unwrap();
        socket.set_rcvtimeo(RECV_TIMEOUT).unwrap();
        socket
            .connect(&common::zmq_url(SOCKET_KIND).unwrap())
            .unwrap();

        socket
    }
}

pub mod project {
    use super::CmdResult;
    use crate::error::server::{LoadUserProjects as LoadUserProjectsError, Update as UpdateError};
    use crate::ProjectCommand as Command;
    use serde_json::Value as JsValue;
    use std::path::PathBuf;
    use syre_core::project::Project;
    use syre_core::types::ResourceId;
    use syre_local::error::IoSerde as IoSerdeError;
    use syre_local::types::ProjectSettings;

    pub struct Client {
        zmq_context: zmq::Context,
    }

    impl Client {
        pub fn new(zmq_context: zmq::Context) -> Self {
            Self { zmq_context }
        }

        pub fn load(&self, path: impl Into<PathBuf>) -> CmdResult<Project, IoSerdeError> {
            let res = self.send(Command::Load(path.into()))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn load_with_settings(
            &self,
            path: impl Into<PathBuf>,
        ) -> CmdResult<(Project, ProjectSettings), IoSerdeError> {
            let res = self.send(Command::LoadWithSettings(path.into()))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn load_user(
            &self,
            user: ResourceId,
        ) -> CmdResult<Vec<(Project, ProjectSettings)>, LoadUserProjectsError> {
            let projects = self.send(Command::LoadUser(user))?;
            Ok(serde_json::from_value(projects).unwrap())
        }

        pub fn get(&self, project: ResourceId) -> zmq::Result<Option<Project>> {
            let project = self.send(Command::Get(project))?;
            Ok(serde_json::from_value(project).unwrap())
        }

        pub fn update(&self, project: Project) -> CmdResult<(), UpdateError> {
            let res = self.send(Command::Update(project))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn path(&self, project: ResourceId) -> zmq::Result<Option<PathBuf>> {
            let res = self.send(Command::GetPath(project))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn resource_root_path(&self, path: PathBuf) -> zmq::Result<Option<PathBuf>> {
            let res = self.send(Command::ResourceRootPath(path))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        fn send(&self, cmd: Command) -> zmq::Result<JsValue> {
            let socket = super::Client::socket(&self.zmq_context);
            let cmd: crate::Command = cmd.into();
            socket.send(&serde_json::to_string(&cmd).unwrap(), 0)?;

            let mut msg = zmq::Message::new();
            socket.recv(&mut msg, 0)?;
            Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
        }
    }
}

pub mod graph {
    use super::CmdResult;
    use crate::error::server::LoadProjectGraph as LoadProjectGraphError;
    use crate::GraphCommand as Command;
    use serde_json::Value as JsValue;
    use syre_core::error::Resource as ResourceError;
    use syre_core::graph::ResourceTree;
    use syre_core::project::Container;
    use syre_core::types::ResourceId;
    use syre_local::error::IoSerde as IoSerdeError;

    pub type ContainerTree = ResourceTree<Container>;

    pub struct Client {
        zmq_context: zmq::Context,
    }

    impl Client {
        pub fn new(zmq_context: zmq::Context) -> Self {
            Self { zmq_context }
        }

        pub fn load(&self, project: ResourceId) -> CmdResult<ContainerTree, LoadProjectGraphError> {
            let res = self.send(Command::Load(project))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn get_or_load(
            &self,
            project: ResourceId,
        ) -> CmdResult<ContainerTree, LoadProjectGraphError> {
            let res = self.send(Command::GetOrLoad(project))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn get(&self, root: ResourceId) -> zmq::Result<Option<ContainerTree>> {
            let res = self.send(Command::Get(root))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn duplicate(&self, root: ResourceId) -> CmdResult<ContainerTree, crate::Error> {
            let res = self.send(Command::Duplicate(root))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn children(
            &self,
            parent: ResourceId,
        ) -> CmdResult<indexmap::IndexSet<Container>, ResourceError> {
            let res = self.send(Command::Children(parent))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn parent(&self, child: ResourceId) -> CmdResult<Option<Container>, ResourceError> {
            let res = self.send(Command::Parent(child))?;
            Ok(serde_json::from_value(res).unwrap())
        }

        fn send(&self, cmd: Command) -> zmq::Result<JsValue> {
            let socket = super::Client::socket(&self.zmq_context);
            let cmd: crate::Command = cmd.into();
            socket.send(&serde_json::to_string(&cmd).unwrap(), 0)?;

            let mut msg = zmq::Message::new();
            socket.recv(&mut msg, 0)?;
            Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
        }
    }
}

pub mod container {
    use super::CmdResult;
    use crate::command::container::{
        AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs,
        BulkUpdatePropertiesArgs, PropertiesUpdate, UpdateAnalysisAssociationsArgs,
        UpdatePropertiesArgs,
    };
    use crate::error::server::{Update as UpdateError, UpdateContainer as UpdateContainerError};
    use crate::ContainerCommand as Command;
    use serde_json::Value as JsValue;
    use std::path::PathBuf;
    use syre_core::project::container::AnalysisMap;
    use syre_core::project::{Container, ContainerProperties};
    use syre_core::types::ResourceId;

    pub struct Client {
        zmq_context: zmq::Context,
    }

    impl Client {
        pub fn new(zmq_context: zmq::Context) -> Self {
            Self { zmq_context }
        }

        pub fn get(&self, container: ResourceId) -> zmq::Result<Option<Container>> {
            let container = self.send(Command::Get(container))?;
            Ok(serde_json::from_value(container).unwrap())
        }

        pub fn path(&self, container: ResourceId) -> zmq::Result<Option<PathBuf>> {
            let path = self.send(Command::Path(container))?;
            Ok(serde_json::from_value(path).unwrap())
        }

        pub fn update_properties(
            &self,
            container: ResourceId,
            properties: ContainerProperties,
        ) -> CmdResult<(), UpdateContainerError> {
            let res = self.send(Command::UpdateProperties(UpdatePropertiesArgs {
                rid: container,
                properties,
            }))?;

            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn update_analysis_associations(
            &self,
            container: ResourceId,
            associations: AnalysisMap,
        ) -> CmdResult<(), UpdateError> {
            let res = self.send(Command::UpdateAnalysisAssociations(
                UpdateAnalysisAssociationsArgs {
                    rid: container,
                    associations,
                },
            ))?;

            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn bulk_update_properties(
            &self,
            containers: Vec<ResourceId>,
            update: PropertiesUpdate,
        ) -> CmdResult<(), crate::Error> {
            let res = self.send(Command::BulkUpdateProperties(BulkUpdatePropertiesArgs {
                rids: containers,
                update,
            }))?;

            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn bulk_update_analysis_associations(
            &self,
            containers: Vec<ResourceId>,
            update: AnalysisAssociationBulkUpdate,
        ) -> CmdResult<(), crate::Error> {
            let res = self.send(Command::BulkUpdateAnalysisAssociations(
                BulkUpdateAnalysisAssociationsArgs { containers, update },
            ))?;

            Ok(serde_json::from_value(res).unwrap())
        }

        fn send(&self, cmd: Command) -> Result<JsValue, zmq::Error> {
            let socket = super::Client::socket(&self.zmq_context);
            let cmd: crate::Command = cmd.into();
            socket.send(&serde_json::to_string(&cmd).unwrap(), 0)?;

            let mut msg = zmq::Message::new();
            socket.recv(&mut msg, 0)?;
            Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
        }
    }
}

pub mod asset {
    use super::CmdResult;
    use crate::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
    use crate::error::server::Update as UpdateError;
    use crate::AssetCommand as Command;
    use serde_json::Value as JsValue;
    use std::path::PathBuf;
    use syre_core::project::{Asset, AssetProperties};
    use syre_core::types::ResourceId;

    pub struct Client {
        zmq_context: zmq::Context,
    }

    impl Client {
        pub fn new(zmq_context: zmq::Context) -> Self {
            Self { zmq_context }
        }

        pub fn get_many(&self, assets: Vec<ResourceId>) -> zmq::Result<Vec<Asset>> {
            let assets = self.send(Command::GetMany(assets))?;
            Ok(serde_json::from_value(assets).unwrap())
        }

        pub fn update_properties(
            &self,
            asset: ResourceId,
            properties: AssetProperties,
        ) -> CmdResult<(), UpdateError> {
            let res = self.send(Command::UpdateProperties { asset, properties })?;
            Ok(serde_json::from_value(res).unwrap())
        }

        pub fn remove(
            &self,
            asset: ResourceId,
        ) -> CmdResult<Option<(Asset, PathBuf)>, crate::Error> {
            let asset_info = self.send(Command::Remove(asset))?;
            Ok(serde_json::from_value(asset_info).unwrap())
        }

        pub fn path(&self, asset: ResourceId) -> zmq::Result<Option<PathBuf>> {
            let path = self.send(Command::Path(asset))?;
            Ok(serde_json::from_value(path).unwrap())
        }

        pub fn bulk_update_properties(
            &self,
            assets: Vec<ResourceId>,
            update: PropertiesUpdate,
        ) -> CmdResult<(), crate::Error> {
            let res = self.send(Command::BulkUpdateProperties(BulkUpdatePropertiesArgs {
                rids: assets,
                update,
            }))?;

            Ok(serde_json::from_value(res).unwrap())
        }

        fn send(&self, cmd: Command) -> Result<JsValue, zmq::Error> {
            let socket = super::Client::socket(&self.zmq_context);
            let cmd: crate::Command = cmd.into();
            socket.send(&serde_json::to_string(&cmd).unwrap(), 0)?;

            let mut msg = zmq::Message::new();
            socket.recv(&mut msg, 0)?;
            Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
        }
    }
}
