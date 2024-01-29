//! Actor for listening to database updates.
use syre_local_database::event::Update;

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
        zmq_socket
            .set_subscribe(syre_local_database::constants::PUB_SUB_TOPIC.as_bytes())
            .unwrap();

        zmq_socket
            .connect(&syre_local_database::common::zmq_url(zmq::SUB).unwrap())
            .unwrap();

        Self { window, zmq_socket }
    }

    /// Instruct the actor to respond to events.
    pub fn run(&self) {
        self.listen_for_events();
    }

    /// Listen for database updates and send them to main window.
    #[tracing::instrument(skip(self))]
    fn listen_for_events(&self) {
        tracing::debug!("listening for file system events");
        loop {
            let messages = match self.zmq_socket.recv_multipart(0) {
                Ok(msg) => msg,
                Err(err) => {
                    tracing::debug!(?err);
                    continue;
                }
            };

            let messages = messages
                .into_iter()
                .map(|msg| zmq::Message::try_from(msg).unwrap())
                .collect::<Vec<_>>();

            let topic = messages
                .get(0)
                .unwrap()
                .as_str()
                .unwrap()
                .replace("local-database", "database/update");

            let mut message = String::new();
            for msg in messages.iter().skip(1) {
                message.push_str(msg.as_str().unwrap());
            }

            let event: Update = serde_json::from_str(&message).unwrap();
            tracing::debug!(?event);
            self.window.emit(&topic, event).unwrap();
        }
    }
}
