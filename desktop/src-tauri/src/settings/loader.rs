//! User settings loader.
use serde::{de::DeserializeOwned, Serialize};
use settings_manager::user_settings::{Loader as UserLoader, UserSettings};
use settings_manager::Result;
use std::marker::PhantomData;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::{HasUser, UserSettingsFile};

pub struct Loader<S>(PhantomData<S>);
impl<S> Loader<S>
where
    S: Serialize + DeserializeOwned + HasUser + Clone,
{
    #[tracing::instrument]
    pub fn load_or_create_with<T>(user: &ResourceId) -> Result<UserLoader<S>>
    where
        T: UserSettings<S> + UserSettingsFile,
    {
        let mut path = PathBuf::from(user.to_string());
        path.push(T::settings_file());

        let default = S::new(user.clone());
        let loader = UserLoader::load_or_create_with::<T>(path, default)?;

        Ok(loader)
    }
}

#[cfg(test)]
#[path = "./loader_test.rs"]
mod loader_test;
