//! Authentication state.
use gloo_storage::{LocalStorage, Storage};
use std::rc::Rc;
use syre_core::system::User;
use yew::prelude::*;

// TODO[h]: App state and auth state should be split in two
// so user is always guaranteed in components.
#[derive(Debug)]
pub enum AuthStateAction {
    // TODO: User should not be Option.
    /// Set the active user.
    SetUser(Option<User>),

    /// Unsets the active user.
    UnsetUser,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct AuthState {
    /// Active user.
    pub user: Option<User>,
}

impl AuthState {
    /// Returns whether a user is authenticated or not.
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }
}

impl Reducible for AuthState {
    type Action = AuthStateAction;

    #[tracing::instrument(level = "debug", skip(self))]
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();

        match action {
            AuthStateAction::SetUser(user) => {
                // store user
                let user_email = match &user {
                    None => None,
                    Some(u) => Some(u.email.clone()),
                };

                // @todo: Maybe not needed to store user in local storage as
                //      it is already stored in system settings.
                //      See also `AppStateAction::UnsetUser`.
                let store_res = LocalStorage::set("user", user_email);
                if let Err(err) = store_res {
                    // TODO Alert user could not store log in.
                    web_sys::console::debug_1(&format!("Could not store user: {:#?}", err).into());
                }

                current.user = user;
            }
            AuthStateAction::UnsetUser => {
                LocalStorage::delete("user");
                current.user = None;
            }
        };

        current.into()
    }
}

pub type AuthStateReducer = UseReducerHandle<AuthState>;
