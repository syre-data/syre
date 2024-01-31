//! User hook to get the current user.
use crate::app::AuthStateReducer;
use syre_core::system::User;
use yew::prelude::*;

// TODO: Should not need to clone user.
/// Gets the current user.
#[hook]
pub fn use_user() -> UseStateHandle<Option<User>> {
    let auth_state = use_context::<AuthStateReducer>().unwrap();
    let user = use_state(|| auth_state.user.clone());
    use_effect_with(auth_state.user.clone(), {
        let user = user.setter();
        move |user_val| {
            user.set(user_val.clone());
        }
    });

    user
}
