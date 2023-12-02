use crate::common;
use crate::server::Event;
use crate::{Command, Error, Result};
use std::sync::mpsc;

/// Actor to handle command events.
pub struct CommandActor {
    event_tx: mpsc::Sender<Event>,

    /// Reply socket for command requests.
    zmq_socket: zmq::Socket,
}

impl CommandActor {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        let zmq_context = zmq::Context::new();
        let zmq_socket = zmq_context.socket(zmq::REP).unwrap();
        zmq_socket
            .bind(&common::zmq_url(zmq::REP).unwrap())
            .unwrap();

        Self {
            event_tx,
            zmq_socket,
        }
    }

    pub fn run(&self) -> Result {
        self.listen_for_commands()?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn listen_for_commands(&self) -> Result {
        loop {
            let cmd = self.receive_command()?;
            let (value_tx, value_rx) = mpsc::channel();
            self.event_tx
                .send(Event::Command { cmd, tx: value_tx })
                .unwrap();

            let res = value_rx.recv().unwrap();
            self.zmq_socket.send(&res.to_string(), 0)?;
        }
    }

    fn receive_command(&self) -> Result<Command> {
        let mut msg = zmq::Message::new();
        self.zmq_socket
            .recv(&mut msg, 0)
            .expect("could not recieve request");

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
}
