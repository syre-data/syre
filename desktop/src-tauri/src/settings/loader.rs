//! User settings loader.
use serde::{de::DeserializeOwned, Serialize};
use settings_manager::user_settings::{Loader as UserLoader, UserSettings};
use settings_manager::Result;
use std::marker::PhantomData;
use std::path::PathBuf;
use thot_core::types::ResourceId;

pub trait UserSettingsFile {
    /// Returns the path to the settings file relative to the user's config directory.
    /// The file should reside at <config_dir>/<users_dir>/<settings_file>.
    fn settings_file() -> PathBuf;
}

pub struct Loader<S>(PhantomData<S>);
impl<S> Loader<S>
where
    S: Serialize + DeserializeOwned + Default + Clone,
{
    pub fn load_or_create<T>(user: &ResourceId) -> Result<UserLoader<S>>
    where
        T: UserSettings<S> + UserSettingsFile,
    {
        let mut path = PathBuf::from(user.to_string());
        path.push(T::settings_file());

        UserLoader::load_or_create::<T>(path)
    }
}

#[cfg(test)]
#[path = "./loader_test.rs"]
mod loader_test;
