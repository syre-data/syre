#![feature(assert_matches)]

use crossbeam::channel::Sender;
use rand::Rng;
use std::{assert_matches::assert_matches, fs, path::PathBuf, thread, time::Duration, u16};
use syre_core::system::User;
use syre_local::error::IoSerde;
use syre_local_database::{
    event, query,
    server::{state, Config},
    types::PortNumber,
    Update,
};

const RECV_TIMEOUT: Duration = Duration::from_millis(500);

#[test_log::test]
fn test_server_state_and_updates() {
    let mut rng = rand::thread_rng();
    let dir = tempfile::tempdir().unwrap();
    let user_manifest = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
    let project_manifest = tempfile::NamedTempFile::new_in(dir.path()).unwrap();

    let config = Config::new(
        user_manifest.path(),
        project_manifest.path(),
        rng.gen_range(1024..u16::max_value()),
    );

    let (update_tx, update_rx) = crossbeam::channel::unbounded();
    let update_listener = UpdateListener::new(update_tx, config.update_port());
    thread::spawn(move || update_listener.run());

    let db = syre_local_database::server::Builder::new(config);
    thread::spawn(move || db.run().unwrap());

    let db = syre_local_database::Client::new();

    let user_manifest_state = db.send(query::State::UserManifest.into()).unwrap();
    let user_manifest_state: state::config::ManifestState<User> =
        serde_json::from_value(user_manifest_state).unwrap();
    assert_matches!(user_manifest_state, Err(IoSerde::Serde(_)));

    let project_manifest_state = db.send(query::State::ProjectManifest.into()).unwrap();
    let project_manifest_state: state::config::ManifestState<PathBuf> =
        serde_json::from_value(project_manifest_state).unwrap();
    assert_matches!(project_manifest_state, Err(IoSerde::Serde(_)));

    // TODO: Handle user manifest
    // fs::write(user_manifest.path(), "{}").unwrap();
    // let update = update_rx.recv_timeout(RECV_TIMEOUT).unwrap();
    // assert_eq!(update.len(), 1);
    // assert_matches!(
    //     update[0].kind(),
    //     event::UpdateKind::App(event::App::UserManifest(event::UserManifest::Repaired))
    // );

    fs::write(project_manifest.path(), "[]").unwrap();
    let update = update_rx.recv_timeout(RECV_TIMEOUT).unwrap();
    assert_eq!(update.len(), 1);
    assert_matches!(
        update[0].kind(),
        event::UpdateKind::App(event::App::ProjectManifest(
            event::ProjectManifest::Repaired
        ))
    );
}

struct UpdateListener {
    tx: Sender<Vec<Update>>,
    socket: zmq::Socket,
}

impl UpdateListener {
    pub fn new(tx: Sender<Vec<Update>>, port: PortNumber) -> Self {
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::SUB).unwrap();
        socket
            .set_subscribe(syre_local_database::constants::PUB_SUB_TOPIC.as_bytes())
            .unwrap();

        socket
            .connect(&syre_local_database::common::localhost_with_port(port))
            .unwrap();

        Self { tx, socket }
    }

    pub fn run(&self) {
        loop {
            let messages = self.socket.recv_multipart(0).unwrap();
            let messages = messages
                .into_iter()
                .map(|msg| zmq::Message::try_from(msg).unwrap())
                .collect::<Vec<_>>();

            let mut message = String::new();
            // skip one for topic
            for msg in messages.iter().skip(1) {
                let msg = msg.as_str().unwrap();
                message.push_str(msg);
            }

            let events: Vec<Update> = serde_json::from_str(&message).unwrap();
            self.tx.send(events).unwrap();
        }
    }
}

mod common {
    use std::fs;
    use std::path::PathBuf;
    use syre_local::project::project;
    use syre_local::project::resources::{Container as LocalContainer, Project as LocalProject};

    pub fn init_project() -> PathBuf {
        let project_dir = tempfile::tempdir().unwrap();
        project::init(project_dir.path()).unwrap();
        project_dir.into_path()
    }

    pub fn init_project_graph(prj: LocalProject) {
        fs::create_dir(prj.data_root_path()).unwrap();
        let root = LocalContainer::new(prj.data_root_path());
        root.save().unwrap();
    }
}
