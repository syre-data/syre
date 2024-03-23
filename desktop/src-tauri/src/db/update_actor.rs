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
    fn listen_for_events(&self) {
        tracing::debug!("listening for file system events");
        'main: loop {
            let messages = match self.zmq_socket.recv_multipart(0) {
                Ok(msg) => msg,
                Err(err) => {
                    tracing::error!(?err);
                    continue;
                }
            };

            let messages = messages
                .into_iter()
                .map(|msg| zmq::Message::try_from(msg).unwrap())
                .collect::<Vec<_>>();

            let Some(topic) = messages.get(0) else {
                tracing::error!("could not get topic from message {messages:?}");
                continue;
            };

            let Some(topic) = topic.as_str() else {
                tracing::error!("could not convert topic to str");
                continue;
            };

            let topic = topic.replace("local-database", "database/update");

            let mut message = String::new();
            for msg in messages.iter().skip(1) {
                let Some(msg) = msg.as_str() else {
                    tracing::error!("could not convert message to str");
                    continue 'main;
                };

                message.push_str(msg);
            }

            let events: Vec<Update> = match serde_json::from_str(&message) {
                Ok(events) => events,
                Err(err) => {
                    tracing::error!(?err);
                    continue;
                }
            };

            tracing::debug!(?events);
            if let Err(err) = self.window.emit(&topic, events) {
                tracing::error!(?err);
            }
        }
    }
}
