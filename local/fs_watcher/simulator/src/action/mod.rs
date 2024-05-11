pub mod app;
pub mod fs;
pub mod watcher;

#[derive(derive_more::From, Debug)]
pub enum Action {
    Watcher(watcher::Action),
    Fs(fs::Action),
}
