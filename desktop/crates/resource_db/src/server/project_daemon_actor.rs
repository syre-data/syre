//! Actor for listening to project updates.
use syre_project_daemon as project_daemon;
use tokio::sync::mpsc;

/// Builder for [`Actor`].
pub struct Builder {
    event_tx: mpsc::UnboundedSender<Vec<project_daemon::Update>>,
}

impl Builder {
    pub fn new(event_tx: mpsc::UnboundedSender<Vec<project_daemon::Update>>) -> Self {
        Self { event_tx }
    }

    /// Create a new actor that listens to database updates.
    /// The actor immediately begins listening.
    pub fn run(self) {
        let zmq_context = zmq::Context::new();
        let zmq_socket = zmq_context.socket(zmq::SUB).unwrap();
        zmq_socket
            .set_subscribe(project_daemon::constants::PUB_SUB_TOPIC.as_bytes())
            .unwrap();

        zmq_socket
            .connect(&project_daemon::common::zmq_url(zmq::SUB).unwrap())
            .unwrap();

        let actor = Actor {
            zmq_socket,
            event_tx: self.event_tx,
        };
        actor.run()
    }
}

/// Actor that listens to and handles updates published from
/// a syre local database.
pub struct Actor {
    /// Socket to listen for updates on.
    zmq_socket: zmq::Socket,

    event_tx: mpsc::UnboundedSender<Vec<project_daemon::Update>>,
}

impl Actor {
    /// Listen for database updates and send them to main window.
    fn run(&self) {
        if !project_daemon::Client::server_available() {
            panic!("`syre-project-daemon` not available");
        }

        'main: loop {
            let messages = match self.zmq_socket.recv_multipart(0) {
                Ok(msg) => msg,
                Err(err) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not receive messages: {err:?}");
                    continue;
                }
            };

            let messages = messages
                .into_iter()
                .map(|msg| zmq::Message::try_from(msg).unwrap())
                .collect::<Vec<_>>();

            let Some(topic) = messages.get(0) else {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not get topic from message {messages:?}");
                continue;
            };

            let Some(topic) = topic.as_str() else {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not convert topic to str");
                continue;
            };

            let mut message = String::new();
            for msg in messages.iter().skip(1) {
                let Some(msg) = msg.as_str() else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not convert message to str");
                    continue 'main;
                };

                message.push_str(msg);
            }

            let updates: Vec<project_daemon::Update> = match serde_json::from_str(&message) {
                Ok(events) => events,
                Err(err) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not convert message to json: `{message}`; {err:?}");
                    continue;
                }
            };

            #[cfg(feature = "tracing")]
            tracing::debug!(?updates);
            if let Err(_) = self.event_tx.send(updates) {
                break;
            };
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("shutting down");
    }
}
