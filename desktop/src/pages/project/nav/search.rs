use super::super::state;
use crate::types;
use leptos::prelude::*;
use reactive_stores::Store;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_resource_db as db;

#[component]
pub fn Search() -> impl IntoView {
    let project = expect_context::<state::Project>();
    let user_settings = expect_context::<Store<types::settings::User>>();
    let input_debounce = user_settings.with(|settings| {
        let debounce = match &settings.desktop {
            Ok(settings) => settings.input_debounce_ms,
            Err(_) => lib::settings::user::Desktop::default().input_debounce_ms,
        };

        debounce as f64
    });
    let (input, set_input) = signal("".to_string());
    let input: Signal<String> = leptos_use::signal_debounced(input, input_debounce);
    let (results, set_results) = signal(db::SearchResult::empty());

    let search_action: Action<_, _> = Action::new_unsync({
        let project = project.path().read_only();
        move |query: &String| {
            let query = query.clone();
            async move {
                let results = search(query, project.get_untracked()).await;
                tracing::trace!(?results);
                set_results(results);
            }
        }
    });

    let _ = Effect::watch(
        move || input.get(),
        move |search, prev_search, _| {
            if let Some(prev_search) = prev_search {
                if search.trim() == prev_search.trim() {
                    return;
                }
            }

            search_action.dispatch(search.trim().to_string());
        },
        false,
    );

    view! {
        <div>
            <form>
                <input class="input-simple" bind:value=(input, set_input) />
            </form>
            <Show when=move || { !input.read().is_empty() } fallback=EmptySearch>
                <SearchResults results />
            </Show>
        </div>
    }
}

#[component]
fn EmptySearch() -> impl IntoView {
    view! { <div>"Search for resources"</div> }
}

#[component]
fn SearchResults(results: ReadSignal<db::SearchResult>) -> impl IntoView {
    view! {
        <Show when=move || { !results.read().is_empty() } fallback=NoMatches>
            <div>
                <div>
                    <h3>"Containers"</h3>
                    <ol>
                        <For
                            each=move || results.with(|results| results.containers().clone())
                            key=|container| container.clone()
                            let:container
                        >
                            <li>
                                <Container rid=container />
                            </li>
                        </For>
                    </ol>
                </div>
                <div>
                    <h3>"Assets"</h3>
                    <ol>
                        <For
                            each=move || results.with(|results| results.assets().clone())
                            key=|asset| asset.clone()
                            let:asset
                        >
                            <li>
                                <Asset rid=asset />
                            </li>
                        </For>
                    </ol>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn NoMatches() -> impl IntoView {
    view! { <div>"(no matches)"</div> }
}

#[component]
fn Container(rid: ResourceId) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let container = graph.nodes().with_untracked(|nodes| {
        nodes
            .iter()
            .find(|node| {
                node.properties().with_untracked(|properties| {
                    let Ok(properties) = properties else {
                        return false;
                    };

                    properties
                        .rid()
                        .with_untracked(|container_rid| *container_rid == rid)
                })
            })
            .unwrap()
            .clone()
    });

    let name = container
        .properties()
        .with_untracked(|properties| properties.as_ref().unwrap().name().read_only());

    let path = {
        let container = container.clone();
        move || {
            graph
                .path(&container)
                .unwrap()
                .to_string_lossy()
                .to_string()
        }
    };

    view! { <div title=path>{name.get()}</div> }
}

#[component]
fn Asset(rid: ResourceId) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let (container, asset) = graph.nodes().with_untracked(|nodes| {
        nodes
            .iter()
            .find_map(|node| {
                node.assets().with_untracked(|assets| {
                    let Ok(assets) = assets else {
                        return None;
                    };

                    assets
                        .read_untracked()
                        .iter()
                        .find(|asset| *asset.rid().read_untracked() == rid)
                        .map(|asset| (node.clone(), asset.clone()))
                })
            })
            .unwrap()
    });

    let name = {
        let name = asset.name().read_only();
        let path = asset.path().read_only();
        move || {
            name.with(|name| {
                if let Some(name) = name.as_ref() {
                    name.clone()
                } else {
                    path.read().to_string_lossy().to_string()
                }
            })
        }
    };

    let path = {
        let container = container.clone();
        move || {
            let path = graph.path(&container).unwrap();

            asset
                .path()
                .with(|asset_path| path.join(asset_path))
                .to_string_lossy()
                .to_string()
        }
    };

    view! { <div title=path>{name()}</div> }
}

async fn search(query: String, project: PathBuf) -> db::SearchResult {
    #[derive(serde::Serialize)]
    struct Args {
        query: String,
        project: PathBuf,
    }

    // TODO: Underlying `search_project` command unecessarily returns a result.
    tauri_sys::core::invoke_result::<db::SearchResult, ()>(
        "search_project",
        Args { query, project },
    )
    .await
    .unwrap()
}
