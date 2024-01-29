//! Main application state.
use gloo_timers::callback::Timeout;
use std::rc::Rc;
use syre_core::project::Project;
use syre_desktop_lib::settings::{UserAppState, UserSettings};
use syre_ui::types::Message;
use uuid::Uuid;
use yew::prelude::*;

/// Application widgets.
#[derive(Debug, Clone, PartialEq)]
pub enum AppWidget {
    /// Create a new project.
    CreateProject,

    /// Intiailize an existing folder as a project.
    InitializeProject,

    /// Import an existing project.
    ImportProject,

    DeleteProject(Project),

    /// User settings.
    UserSettings,
}

/// Actions available to modify the [`AppState`].
#[derive(Debug)]
pub enum AppStateAction {
    /// Sets the active widget.
    SetActiveWidget(Option<AppWidget>),

    /// Add a message to display.
    AddMessage(Message),

    // TODO Remove requirement to pass `AppStateReducer`.
    /// Adda a message to display,
    /// disappering after some time.
    AddMessageWithTimeout(Message, u32, AppStateReducer<'static>),

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
    /// `([Message], timeout).
    pub messages: Vec<Rc<Message>>,

    /// User's application state.
    pub user_app_state: Option<UserAppState>,

    /// User's settings.
    pub user_settings: Option<UserSettings>,
}

impl Reducible for AppState {
    type Action = AppStateAction;

    #[tracing::instrument(skip(self))]
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            AppStateAction::SetActiveWidget(widget) => {
                current.app_widget = widget;
            }

            AppStateAction::AddMessage(message) => {
                current.messages.push(Rc::new(message));
            }

            AppStateAction::AddMessageWithTimeout(message, timeout, state) => {
                let mid = message.id().clone();

                current.messages.push(Rc::new(message));
                let timeout = Timeout::new(timeout, move || {
                    state.dispatch(AppStateAction::RemoveMessage(mid));
                });

                timeout.forget();
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

pub type AppStateReducer<'a> = UseReducerHandle<AppState>;
pub type AppStateDispatcher = UseReducerDispatcher<AppState>;
