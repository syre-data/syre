pub enum Event {
    Command {
        cmd: crate::Command,
        tx: std::sync::mpsc::Sender<serde_json::Value>,
    },

    FileSystem(notify_debouncer_full::DebounceEventResult),
}
