use super::super::{
    common::{SelectionAction, interpret_resource_selection_action},
    state,
};
use crate::types;
use leptos::{ev::MouseEvent, prelude::*};
use reactive_stores::Store;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_resource_db as db;

/// Stores search history.
///
/// History is stored in choronological order (i.e. Older terms before newer).
/// A term may only appear once. If a term already exists in the history and is `push`ed
/// in, the existing element is removed and placed at the end of the list
/// (i.e. it becomes the newest element).
///
/// # Example
/// ```rust
/// let mut history = SearchHistory::new();
/// history.push("a"); // ["a"]
/// history.push("b"); // ["a", "b"]
/// history.push("c"); // ["a", "b", "c"]
/// history.push("b"); // ["a", "c", "b"]
/// ```
#[derive(Clone)]
struct SearchHistory {
    inner: Vec<String>,
}
impl SearchHistory {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn push(&mut self, term: String) {
        if let Some(idx) = self.inner.iter().position(|history| *history == term) {
            self.inner.remove(idx);
            self.inner.push(term);
        } else {
            self.inner.push(term);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::iter::IntoIterator for SearchHistory {
    type Item = String;
    type IntoIter = std::iter::Rev<std::vec::IntoIter<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().rev()
    }
}

#[component]
pub fn Search() -> impl IntoView {
    let project = expect_context::<state::Project>();
    let user_settings = expect_context::<Store<types::settings::User>>();
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
    let (results, set_results) = signal(db::SearchResult::empty());
    let (history, set_history) = signal(SearchHistory::new());

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
            search_action.dispatch(search.clone());
            set_history.write().push(search);
        },
        false,
    );

    view! {
        <div>
            <form>
                <div class="p-1">
                    <input
                        bind:value=(input, set_input)
                        class="input-compact"
                        placeholder="Search"
                    />
                </div>
            </form>
            <Show
                when=move || { !input.read().is_empty() }
                fallback=move || view! { <SearchHistory history set_query set_input /> }
            >
                <SearchResults results />
            </Show>
        </div>
    }
}

#[component]
fn SearchHistory(
    history: ReadSignal<SearchHistory>,
    set_query: WriteSignal<String>,
    set_input: WriteSignal<String>,
) -> impl IntoView {
    let mousedown = move |e: MouseEvent, term: String| {
        if e.button() != types::MouseButton::Primary {
            return;
        }

        // NB: `set_query` must be called before `set_input`
        // So action is not called twice.
        // See the above effect watching `input` for how this
        // is prevented.
        set_query(term.clone());
        set_input(term);
    };

    view! {
        <Show when=move || !history.read().is_empty() fallback=EmptySearch>
            <div class="px-2">
                <div class="font-primary">"Previous searches"</div>
                <ol class="overflow-y-auto text-sm">
                    <For each=history key=|term| term.clone() let:term>
                        <li
                            on:mousedown=move |e| mousedown(e, term.clone())
                            class="cursor-pointer hover:text-primary-600"
                        >
                            {term.clone()}
                        </li>
                    </For>
                </ol>
            </div>
        </Show>
    }
}

#[component]
fn EmptySearch() -> impl IntoView {
    view! { <div class="text-center">"Search for resources"</div> }
}

#[component]
fn SearchResults(results: ReadSignal<db::SearchResult>) -> impl IntoView {
    view! {
        <Show when=move || { !results.read().is_empty() } fallback=NoMatches>
            <div class="overflow-auto">
                <div class="text-sm/4">
                    <h3 class="text-base font-primary px-1">"Containers"</h3>
                    <Show
                        when=move || !results.read().containers().is_empty()
                        fallback=NoResourceMatches
                    >
                        <ol>
                            <For
                                each=move || results.read().containers().clone()
                                key=|container| container.clone()
                                let:container
                            >
                                <li>
                                    <Container rid=container />
                                </li>
                            </For>
                        </ol>
                    </Show>
                </div>
                <div class="text-sm/4">
                    <h3 class="text-base font-primary px-1 pt-1">"Assets"</h3>
                    <Show
                        when=move || !results.read().assets().is_empty()
                        fallback=NoResourceMatches
                    >
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
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn NoMatches() -> impl IntoView {
    view! { <div class="text-center">"(no matches)"</div> }
}

#[component]
fn NoResourceMatches() -> impl IntoView {
    view! { <div class="px-1">"(no matches)"</div> }
}

#[component]
fn Container(rid: ResourceId) -> impl IntoView {
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
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

    let selection_resource = container
        .properties()
        .with_untracked(|properties| {
            let properties = properties.as_ref().unwrap();
            properties.rid().with_untracked(|container_id| {
                workspace_graph_state
                    .selection_resources()
                    .get(container_id)
            })
        })
        .unwrap();

    let mousedown = {
        let rid = container
            .properties()
            .with_untracked(|properties| properties.as_ref().unwrap().rid().read_only());
        let selection_resources = workspace_graph_state.selection_resources().clone();
        move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            let action = rid.with_untracked(|rid| {
                selection_resources.selected().with_untracked(|selected| {
                    interpret_resource_selection_action(rid, selected, e.shift_key())
                })
            });
            match action {
                SelectionAction::Unselect => {
                    rid.with_untracked(|rid| selection_resources.set(rid, false).unwrap())
                }
                SelectionAction::Select => {
                    rid.with_untracked(|rid| selection_resources.set(rid, true).unwrap())
                }
                SelectionAction::SelectOnly => {
                    rid.with_untracked(|rid| selection_resources.select_only(rid).unwrap())
                }
                SelectionAction::Clear => selection_resources.clear(),
            }
        }
    };

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

    view! {
        <div
            on:mousedown=mousedown
            title=path
            class="p-1 cursor-pointer border-y border-y-transparent \
            hover:border-y-black dark:hover:border-y-secondary-400"
            class=(["bg-secondary-100", "dark:bg-secondary-900"], selection_resource.clone())
        >
            {name.get()}
        </div>
    }
}

#[component]
fn Asset(rid: ResourceId) -> impl IntoView {
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
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

    let selection_resource = asset
        .rid()
        .with_untracked(|rid| workspace_graph_state.selection_resources().get(rid))
        .unwrap();

    let mousedown = {
        let selection_resources = workspace_graph_state.selection_resources().clone();
        let rid = asset.rid().read_only();
        move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            let action = rid.with_untracked(|rid| {
                selection_resources.selected().with_untracked(|selected| {
                    interpret_resource_selection_action(rid, selected, e.shift_key())
                })
            });
            match action {
                SelectionAction::Unselect => {
                    rid.with_untracked(|rid| selection_resources.set(rid, false).unwrap())
                }
                SelectionAction::Select => {
                    rid.with_untracked(|rid| selection_resources.set(rid, true).unwrap())
                }
                SelectionAction::SelectOnly => {
                    rid.with_untracked(|rid| selection_resources.select_only(rid).unwrap())
                }
                SelectionAction::Clear => selection_resources.clear(),
            }
        }
    };

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

    view! {
        <div
            on:mousedown=mousedown
            title=path
            class="p-1 cursor-pointer border-y border-y-transparent \
            hover:border-y-black dark:hover:border-y-secondary-400"
            class=(["bg-secondary-100", "dark:bg-secondary-900"], selection_resource.clone())
        >
            {name()}
        </div>
    }
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
