use super::RunParameters;
use super::StandardResource;
use crate::types::{ResourceMap, ResourceStore};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub trait Asset: StandardResource {
    fn path(&self) -> &Path;
    fn set_path(&mut self, path: PathBuf);
}

pub trait Container: StandardResource {
    type Asset: Asset;

    fn children(&self) -> &ResourceStore<Arc<Mutex<Self>>>;
    fn children_mut(&mut self) -> &mut ResourceStore<Arc<Mutex<Self>>>;

    fn assets(&self) -> &ResourceMap<Self::Asset>;
    fn assets_mut(&mut self) -> &mut ResourceMap<Self::Asset>;

    fn scripts(&self) -> &ResourceMap<RunParameters>;
    fn scripts_mut(&mut self) -> &mut ResourceMap<RunParameters>;
}
