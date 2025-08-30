use crate::state;
use leptos::{ev::SubmitEvent, prelude::*};
use reactive_stores::Store;
use std::path::PathBuf;
use syre_desktop_lib as lib;
use syre_desktop_resource_db as db;
use syre_desktop_ui_lib as ui_lib;

#[component]
pub fn DataFilter() -> impl IntoView {
    let display_state = expect_context::<state::display::State>();
    let project = expect_context::<ui_lib::state::Project>();
    let user_settings = expect_context::<Store<ui_lib::state::settings::User>>();
    let input_debounce = Signal::derive(move || {
        user_settings.with(|settings| {
            let debounce = match &settings.desktop {
                Ok(settings) => settings.input_debounce_ms,
                Err(_) => lib::settings::user::Desktop::default().input_debounce_ms,
            };

            debounce as f64
        })
    });
    let (input, set_input) = signal("".to_string());
    let input: Signal<String> = leptos_use::signal_debounced(input, input_debounce);
    let (query, set_query) = signal("".to_string());

    let filter_action: Action<_, _> = Action::new_unsync({
        let project = project.path().read_only();
        let filter = display_state.filter().write_only();
        move |query: &String| {
            let query = query.clone();
            async move {
                if query.trim().is_empty() {
                    filter.set(None);
                } else {
                    let query = query.trim().to_string();

                    #[cfg(feature = "tracing")]
                    tracing::trace!("querying `{query}`");

                    let results = search_assets(query, project.get_untracked()).await;

                    #[cfg(feature = "tracing")]
                    tracing::trace!(?results);

                    filter.set(Some(results.assets().clone()));
                }
            }
        }
    });

    let _ = Effect::watch(
        move || input.get(),
        move |search, _, _| {
            if query.with_untracked(|query| search.trim() != query) {
                set_query(search.trim().to_string());
            }
        },
        false,
    );

    let _ = Effect::watch(
        query,
        move |query, prev_query, _| {
            if let Some(prev_query) = prev_query {
                if query.trim() == prev_query.trim() {
                    return;
                }
            }

            let search = query.trim().to_string();
            filter_action.dispatch(search.clone());
        },
        false,
    );

    view! {
        <form on:submit=move |e: SubmitEvent| e.prevent_default()>
            <div>
                <input
                    class="w-full px-1 py-0.5 leading-5 border-0 dark:text-white bg-transparent focus:ring-0"
                    placeholder="Filter"
                    bind:value=(input, set_input)
                />
            </div>
        </form>
    }
}

async fn search_assets(query: String, project: PathBuf) -> db::AssetSearchResult {
    #[derive(serde::Serialize)]
    struct Args {
        query: String,
        project: PathBuf,
    }

    // TODO: Underlying `search_project_assets` command unecessarily returns a result.
    tauri_sys::core::invoke_result::<db::AssetSearchResult, ()>(
        "search_project_assets",
        Args { query, project },
    )
    .await
    .unwrap()
}
