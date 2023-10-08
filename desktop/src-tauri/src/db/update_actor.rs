//! Actor for listening to database updates.
use std::sync::mpsc;
use std::thread;
use thot_local_database::update::Update;

pub struct UpdateActor {
    window: tauri::Window,
    zmq_socket: zmq::Socket,
}

impl UpdateActor {
    /// Create a new actor that listens to database updates.
    /// The actor immediately begins listening.
    pub fn new(window: tauri::Window) -> Self {
        let zmq_context = zmq::Context::new();
        let zmq_socket = zmq_context.socket(zmq::SUB).unwrap();
        zmq_socket.set_subscribe(thot_local_database::constants::PUB_SUB_TOPIC);
        zmq_socket
            .connect(&thot_local_database::common::zmq_url(zmq::SUB).unwrap())
            .unwrap();

        Self { window, zmq_socket }
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

            let update: Update = serde_json::from_str(message.as_str().unwrap()).unwrap();
            tracing::debug!(?update);
            self.window.emit("thot://database-update", update).unwrap();
        }
    }
}
