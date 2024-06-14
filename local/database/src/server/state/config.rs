//! Application configuration state.
pub use action::Action;
use std::path::PathBuf;
use syre_core::system::User;
use syre_local::{error::IoSerde, Reducible};

pub type ManifestState<T> = Result<Vec<T>, IoSerde>;

/// Application config state.
#[derive(Debug)]
pub struct State {
    /// Users.
    user_manifest: ManifestState<User>,

    /// Project paths.
    project_manifest: ManifestState<PathBuf>,
}

impl State {
    pub fn new(
        user_manifest: ManifestState<User>,
        project_manifest: ManifestState<PathBuf>,
    ) -> Self {
        Self {
            user_manifest,
            project_manifest,
        }
    }

    pub fn user_manifest(&self) -> &ManifestState<User> {
        &self.user_manifest
    }

    pub fn project_manifest(&self) -> &ManifestState<PathBuf> {
        &self.project_manifest
    }
}

impl Reducible for State {
    type Action = Action;
    fn reduce(&mut self, action: Self::Action) {
        match action {
            Action::UserManifest(action) => match action {
                action::Manifest::SetOk(value) => self.user_manifest = ManifestState::Ok(value),
                action::Manifest::SetErr(err) => self.user_manifest = ManifestState::Err(err),
            },
            Action::ProjectManifest(action) => match action {
                action::Manifest::SetOk(value) => self.project_manifest = ManifestState::Ok(value),
                action::Manifest::SetErr(err) => self.project_manifest = ManifestState::Err(err),
            },
        }
    }
}

pub mod action {
    use std::path::PathBuf;
    use syre_core::system::User;
    use syre_local::error::IoSerde;

    #[derive(Debug)]
    pub enum Action {
        UserManifest(Manifest<Vec<User>>),
        ProjectManifest(Manifest<Vec<PathBuf>>),
    }

    #[derive(Debug)]
    pub enum Manifest<T> {
        SetOk(T),
        SetErr(IoSerde),
    }
}
