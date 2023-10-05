//! Actor for listening to database updates.
use std::sync::mpsc;
use std::thread;
use thot_local_database::common;
use thot_local_database::update::Update;

// *************
// *** Actor ***
// *************

pub struct UpdateActor {
    update_tx: mpsc::Sender<Update>,
    zmq_socket: zmq::Socket,
}

impl UpdateActor {
    /// Create a new actor that listens to database updates.
    /// The actor immediately begins listening.
    pub fn new(update_tx: mpsc::Sender<Update>) -> Self {
        let zmq_context = zmq::Context::new();
        let zmq_socket = zmq_context.socket(zmq::SUB).unwrap();
        zmq_socket
            .connect(&common::zmq_url(zmq::SUB).unwrap())
            .unwrap();

        Self {
            update_tx,
            zmq_socket,
        }
    }

    /// Instruct the actor to respond to events.
    pub fn run(&self) {
        self.listen_for_updates();
    }

    /// Listen for database updates and send them over the tx channel.
    #[tracing::instrument(skip(self))]
    fn listen_for_updates(&self) {
        loop {
            let message = match self.zmq_socket.recv_msg(0) {
                Ok(msg) => msg,
                Err(err) => {
                    tracing::debug!(?err);
                    continue;
                }
            };

            let update = serde_json::from_str(message.as_str().unwrap()).unwrap();
            tracing::debug!(?update);
            self.update_tx.send(update).unwrap();
        }
    }
}

// **************
// *** Handle ***
// **************

pub struct UpdateActorHandle {
    update_rx: mpsc::Receiver<Update>,
    window: tauri::Window,
}

impl UpdateActorHandle {
    pub fn new(update_rx: mpsc::Receiver<Update>, window: tauri::Window) -> Self {
        Self { update_rx, window }
    }

    pub fn run(&self) {
        self.handle_database_updates()
    }

    #[tracing::instrument(skip(self))]
    fn handle_database_updates(&self) {
        tracing::debug!("LISTENING");
        loop {
            let update = self.update_rx.recv().unwrap();
            tracing::debug!(?update);
            self.window.emit("thot://database-update", update).unwrap();
        }
    }
}
