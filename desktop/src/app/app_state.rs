//! Main application state.
use std::rc::Rc;
use thot_desktop_lib::settings::{UserAppState, UserSettings};
use thot_ui::types::Message;
use uuid::Uuid;
use yew::prelude::*;

/// Application widgets.
#[derive(Debug, Clone, PartialEq)]
pub enum AppWidget {
    /// Create a new project.
    CreateProject,

    /// User settings.
    UserSettings,
}

/// Actions available to modify the [`AppState`].
pub enum AppStateAction {
    /// Sets the active widget.
    SetActiveWidget(Option<AppWidget>),

    /// Add a message to display.
    AddMessage(Message),

    /// Removes a message.
    RemoveMessage(Uuid),

    /// Clears all messages.
    ClearMessages,

    /// Sets the user's app state.
    SetUserAppState(Option<UserAppState>),

    /// Sets the user's settings.
    SetUserSettings(Option<UserSettings>),

    /// Clears the user app state and settings.
    ClearUserSettingAndAppState,
}

/// Application state.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct AppState {
    /// Active widget
    pub app_widget: Option<AppWidget>,

    /// Messages for the user.
    pub messages: Vec<Rc<Message>>,

    /// User's application state.
    pub user_app_state: Option<UserAppState>,

    /// User's settings.
    pub user_settings: Option<UserSettings>,
}

impl Reducible for AppState {
    type Action = AppStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            AppStateAction::SetActiveWidget(widget) => {
                current.app_widget = widget;
            }
            AppStateAction::AddMessage(message) => {
                current.messages.push(Rc::new(message));
            }
            AppStateAction::RemoveMessage(id) => {
                current.messages.retain(|m| m.id() != &id);
            }
            AppStateAction::ClearMessages => {
                current.messages = Vec::new();
            }
            AppStateAction::SetUserAppState(state) => {
                current.user_app_state = state;
            }
            AppStateAction::SetUserSettings(settings) => {
                current.user_settings = settings;
            }
            AppStateAction::ClearUserSettingAndAppState => {
                current.user_app_state = None;
                current.user_settings = None;
            }
        };

        current.into()
    }
}

pub type AppStateReducer = UseReducerHandle<AppState>;

#[cfg(test)]
#[path = "./app_state_test.rs"]
mod app_state_test;
