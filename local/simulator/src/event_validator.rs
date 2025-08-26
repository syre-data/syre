use crossbeam::channel::{Receiver, Sender, select};
use syre_fs_daemon::{Event, EventResult};

pub struct EventValidator {
    expected: Vec<Event>,
    received: Vec<Event>,
    expected_rx: Receiver<Vec<Event>>,
    daemon_rx: Receiver<EventResult>,
    validation_tx: Sender<error::Validation>,
}

impl EventValidator {
    pub fn new(
        daemon_rx: Receiver<EventResult>,
        expected_rx: Receiver<Vec<Event>>,
        validation_tx: Sender<error::Validation>,
    ) -> Self {
        Self {
            expected: vec![],
            received: vec![],
            expected_rx,
            daemon_rx,
            validation_tx,
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        loop {
            select! {
                recv(self.daemon_rx) -> events => match events {
                    Ok(events) => self.handle_daemon_events(events)?,
                    Err(err) => {
                        tracing::error!("daemon: {err:}");
                        return Err(());
                    }
                },

                recv(self.expected_rx) -> events => match events {
                    Ok(events) => self.handle_expected_events(events),
                    Err(err) => {
                        tracing::error!("simulator: {err:}");
                        return Err(());
                    }
                },

                default => self.validate_events()
            }
        }
    }
}

impl EventValidator {
    fn handle_daemon_events(&mut self, events: EventResult) -> Result<(), ()> {
        match events {
            Ok(mut events) => {
                self.received.append(&mut events);
                Ok(())
            }

            Err(errors) => {
                tracing::error!(?errors);
                Err(())
            }
        }
    }

    fn handle_expected_events(&mut self, mut events: Vec<Event>) {
        self.expected.append(&mut events);
    }
}

impl EventValidator {
    fn validate_events(&self) {}
}

pub mod error {
    use syre_fs_daemon::Event;

    #[derive(Debug)]
    pub struct Validation {
        pub expected: Event,
        pub received: Event,
    }
}
