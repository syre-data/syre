//! App state.
use std::{
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use syre_core::types::ResourceId;

/// App state.
pub struct State {
    /// Active user.
    user: Slice<Option<ResourceId>>,

    /// Project paths associated with the current user.
    projects: Slice<Vec<PathBuf>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            user: Slice::new(None),
            projects: Slice::new(vec![]),
        }
    }

    pub fn user(&self) -> &Slice<Option<ResourceId>> {
        &self.user
    }

    pub fn projects(&self) -> &Slice<Vec<PathBuf>> {
        &self.projects
    }
}

/// Slice of the state.
///
/// Arc<Mutex<T>> newtype for convenience.
pub struct Slice<T>(Arc<Mutex<T>>);
impl<T> Slice<T> {
    fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl<T> Deref for Slice<T> {
    type Target = Arc<Mutex<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
