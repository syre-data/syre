use std::path::PathBuf;

#[derive(Debug)]
pub enum Action {
    Watch(PathBuf),
    Unwatch(PathBuf),
}
