//! User hook to get the current user.
use crate::app::AuthStateReducer;
use thot_core::system::User;
use yew::prelude::*;

/// Gets the current user.
#[hook]
pub fn use_user() -> UseStateHandle<Option<User>> {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let user = use_state(|| auth_state.user.clone());
    {
        let auth_state = auth_state.clone();
        let user = user.clone();

        use_effect_with_deps(
            move |auth_state| {
                user.set(auth_state.user.clone());
            },
            auth_state,
        );
    }

    user
}
