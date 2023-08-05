//! Get the users preferred theme.
use crate::Result;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::prelude::*;

// ***********************
// *** Preferred Theme ***
// ***********************

/// User themes.
#[derive(Clone)]
pub enum PreferredTheme {
    Light,
    Dark,
}

/// Gets the users preferred theme.
pub fn preferred_theme() -> Result<Option<PreferredTheme>> {
    let wnd = web_sys::window().expect("window not found");
    let dark_theme_mq = wnd.match_media("(prefers-color-scheme: dark)")?;
    let Some(dark_theme_mq) = dark_theme_mq else {
        return Ok(None);
    };

    if dark_theme_mq.matches() {
        Ok(Some(PreferredTheme::Dark))
    } else {
        Ok(Some(PreferredTheme::Light))
    }
}

/// Hook for preferred theme.
#[hook]
pub fn use_preferred_theme() -> Option<PreferredTheme> {
    let state = use_state(|| None);

    {
        let state = state.clone();

        use_effect(move || {
            let wnd = web_sys::window().expect("window not found");
            let dark_theme_mq = wnd
                .match_media("(prefers-color-scheme: dark)")
                .expect("could not match media")
                .expect("media query not found");

            let set_theme: Closure<dyn Fn(web_sys::MediaQueryListEvent)> =
                Closure::new(move |e: web_sys::MediaQueryListEvent| {
                    let theme = {
                        if e.matches() {
                            Some(PreferredTheme::Dark)
                        } else {
                            Some(PreferredTheme::Light)
                        }
                    };

                    state.set(theme);
                });

            dark_theme_mq.set_onchange(Some(set_theme.as_ref().unchecked_ref()));

            // clean up
            move || {
                dark_theme_mq
                    .remove_event_listener_with_callback(
                        "change",
                        set_theme.as_ref().unchecked_ref(),
                    )
                    .expect("could not remove event listener");
            }
        });
    }

    (*state).clone()
}
