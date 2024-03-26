//! Search bar.
use super::ResourcesBarWidget;
use crate::commands::search;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use syre_core::types::ResourceId;
use syre_local_database::command::search::{Field, Query, QueryBuilder};
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[function_component(SearchBar)]
pub fn search_bar() -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let input_state = use_state(InputState::default);
    let results = use_state(|| None);

    let search = use_callback((), {
        let results = results.setter();
        move |query: String, _| {
            if query.len() < 2 {
                results.set(None);
                return;
            }

            let results = results.clone();
            spawn_local(async move {
                match search::search(query).await {
                    Ok(resources) => results.set(Some(Ok(SearchResult::new(resources)))),
                    Err(err) => {
                        tracing::error!(?err);
                        results.set(Some(Err("Could not retrieve results.".to_string())));
                    }
                };
            })
        }
    });

    let query = use_callback((), {
        let results = results.setter();
        move |query: Query, _| {
            let results = results.clone();
            spawn_local(async move {
                match search::query(query).await {
                    Ok(resources) => results.set(Some(Ok(SearchResult::new(resources)))),
                    Err(err) => {
                        tracing::error!(?err);
                        results.set(Some(Err("Could not retrieve results.".to_string())));
                    }
                };
            })
        }
    });

    let close_search = use_callback((), {
        let canvas_state = canvas_state.dispatcher();
        move |_e: MouseEvent, _| {
            canvas_state.dispatch(CanvasStateAction::SetResourcesBarWidget(
                ResourcesBarWidget::default(),
            ));
        }
    });

    let search_results = match results.as_ref() {
        Some(Ok(SearchResult { resources })) => resources.clone(),
        _ => vec![],
    };

    html! {
        <div class={"search-bar"}>
            <div class={"search-input"}>
                { match *input_state {
                    InputState::Search => html! {
                        <div class={"search-form"}>
                            <div class={"flex"}>
                                <SearchForm oninput={search} />
                                <div class={"widget-controls"}>
                                    <button type={"button"}
                                        onclick={close_search}>

                                        <Icon icon_id={IconId::FontAwesomeSolidXmark} />
                                    </button>
                                </div>
                            </div>

                            if let Some(Err(err)) = results.as_ref() {
                                <div class={"form-error"}>
                                    {err}
                                </div>
                            }

                            <div>
                                <button class={"text-btn"}
                                    onclick={
                                        let input_state = input_state.setter();
                                        move |_| input_state.set(InputState::Query)
                                    }>

                                    { "advanced search" }
                                </button>
                            </div>
                        </div>
                    },

                    InputState::Query => html! {
                        <div class={"query-form"}>
                            <div class={"widget-controls flex"}>
                                <button class={"text-btn"}
                                    onclick={
                                        let input_state = input_state.setter();
                                        move |_| input_state.set(InputState::Search)
                                    }>

                                    { "basic search" }
                                </button>

                                <button type={"button"}
                                    onclick={close_search}>

                                    <Icon icon_id={IconId::FontAwesomeSolidXmark} />
                                </button>
                            </div>
                            <QueryForm onsubmit={query} />
                        </div>
                    }
                }}
            </div>

            <SearchResults resources={search_results} />
        </div>
    }
}

struct SearchResult {
    resources: Vec<ResourceId>,
}

impl SearchResult {
    fn new(resources: Vec<ResourceId>) -> Self {
        Self { resources }
    }
}

#[derive(PartialEq, Properties)]
struct SearchFormProps {
    oninput: Callback<String>,
}

#[function_component(SearchForm)]
fn search_form(props: &SearchFormProps) -> Html {
    let input_ref = use_node_ref();

    let onsubmit = use_callback(props.oninput.clone(), {
        let input_ref = input_ref.clone();
        move |e: SubmitEvent, oninput| {
            e.prevent_default();

            let input = input_ref.cast::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value();
            oninput.emit(value);
        }
    });

    let oninput = use_callback(props.oninput.clone(), {
        let input_ref = input_ref.clone();
        move |_e: InputEvent, oninput| {
            let input = input_ref.cast::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value();
            oninput.emit(value)
        }
    });

    html! {
        <form class={"grow"}
            {onsubmit}>

            <div class={"flex"}>
                <label>
                    <Icon icon_id={IconId::BootstrapSearch} />
                </label>

                <input ref={input_ref}
                    class={"grow"}
                    placeholder={"Find..."}
                    {oninput} />
            </div>
        </form>
    }
}

#[derive(PartialEq, Properties)]
struct QueryFormProps {
    onsubmit: Callback<Query>,
}

#[function_component(QueryForm)]
fn query_form(props: &QueryFormProps) -> Html {
    let form_ref = use_node_ref();

    let onsubmit = use_callback(props.onsubmit.clone(), {
        let form_ref = form_ref.clone();
        move |e: SubmitEvent, onsubmit| {
            e.prevent_default();

            let form = form_ref.cast::<web_sys::HtmlFormElement>().unwrap();
            let data = web_sys::FormData::new_with_form(&form).unwrap();
            let name = data.get("name").as_string().unwrap();
            let kind = data.get("kind").as_string().unwrap();
            let tags = data.get("tags").as_string().unwrap();
            let tags = tags.split(',').collect::<Vec<_>>();
            let tags = tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .collect::<Vec<_>>();

            let query = QueryBuilder::new(Field::Name(Some(name)));
            onsubmit.emit(query.build());
        }
    });

    html! {
        <form {onsubmit}>
            <div class={"form-control"}>
                <label>{ "Name" }</label>
                <input name={"name"} />
            </div>

            <div class={"form-control"}>
                <label>{ "Type" }</label>
                <input name={"type"} />
            </div>

            <div class={"form-control"}>
                <label>{ "Tags" }</label>
                <input name={"tags"} />
            </div>
        </form>
    }
}

#[derive(PartialEq, Properties)]
struct SearchResultsProps {
    resources: Vec<ResourceId>,
}

#[function_component(SearchResults)]
fn search_results(props: &SearchResultsProps) -> Html {
    html! {
        <ul class={"search-results"}>
            { props.resources.iter().map(|rid| html! { {rid} }).collect::<Vec<_>>() }
        </ul>
    }
}

enum InputState {
    Search,
    Query,
}

impl Default for InputState {
    fn default() -> Self {
        Self::Search
    }
}
