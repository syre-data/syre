//! Collection of widgetst available from anywhere in the app.
//! Activation of the widgets is determined by the [`AppState`].
use crate::app::{AppStateAction, AppStateReducer, AppWidget, ShadowBox};
use crate::components::dashboard::project::{CreateProject, ImportProject};
use crate::components::settings::Settings;
use yew::prelude::*;

#[function_component(GlobalWidgets)]
pub fn global_widgets() -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let deactivate_app_widget = {
        let app_state = app_state.clone();

        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(None));
        })
    };

    let (title, widget) = if let Some(wgt) = &app_state.app_widget {
        match wgt {
            AppWidget::CreateProject => ("Create a project", html! { <CreateProject /> }),
            AppWidget::ImportProject => ("Import project", html! { <ImportProject /> }),
            AppWidget::UserSettings => ("Settings", html! { <Settings /> }),
        }
    } else {
        ("", html! {})
    };

    html! {
        if app_state.app_widget.is_some() {
            <ShadowBox {title} onclose={deactivate_app_widget}>
                { widget }
            </ShadowBox>
        }
    }
}
