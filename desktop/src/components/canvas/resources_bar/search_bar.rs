//! Search bar.
use super::ResourcesBarWidget;
use crate::commands::search;
use crate::components::canvas::{
    canvas_state::ResourceType, CanvasStateAction, CanvasStateReducer, GraphStateReducer,
};
use syre_core::types::ResourceId;
use syre_local_database::command::search::{Field, Query, QueryBuilder};
use syre_ui::constants::ICON_SIZE;
use syre_ui::widgets::common as ui_common;
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
            let query = query.trim().to_string();
            if query.is_empty() {
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
                        <div class={"search-form-wrapper"}>
                            <div class={"flex"}>
                                <SearchForm oninput={search} />
                                <div class={"widget-controls"}>
                                    <button
                                        type={"button"}
                                        onclick={close_search}
                                    >
                                        <Icon
                                            icon_id={IconId::FontAwesomeSolidXmark}
                                            width={ICON_SIZE.to_string()}
                                            height={ICON_SIZE.to_string()}
                                        />
                                    </button>
                                </div>
                            </div>

                            if let Some(Err(err)) = results.as_ref() {
                                <div class={"form-error"}>
                                    {err}
                                </div>
                            }

                            // <div>
                            //     <button class={"text-btn"}
                            //         onclick={
                            //             let input_state = input_state.setter();
                            //             move |_| input_state.set(InputState::Query)
                            //         }>

                            //         { "advanced search" }
                            //     </button>
                            // </div>
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
        <form class={"search-form grow"}
            {onsubmit}>

            <div class={"flex"}>
                <label>
                    <Icon
                        class="align-middle"
                        icon_id={IconId::BootstrapSearch}
                        width={ICON_SIZE.to_string()}
                        height={ICON_SIZE.to_string()}
                    />
                </label>

                <input ref={input_ref}
                    class={"grow"}
                    placeholder={"Find..."}
                    {oninput}
                    autofocus={true}
                />
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
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let toggle_select_container = {
        let canvas_state = canvas_state.dispatcher();
        move |rid: ResourceId| {
            let canvas_state = canvas_state.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();

                canvas_state.dispatch(CanvasStateAction::ToggleSelected {
                    resource: rid.clone(),
                    kind: ResourceType::Container,
                    multiselect: e.shift_key(),
                });
            })
        }
    };

    let toggle_select_asset = {
        let canvas_state = canvas_state.dispatcher();
        move |rid: ResourceId| {
            let canvas_state = canvas_state.clone();
            Callback::from(move |e: MouseEvent| {
                canvas_state.dispatch(CanvasStateAction::ToggleSelected {
                    resource: rid.clone(),
                    kind: ResourceType::Asset,
                    multiselect: e.shift_key(),
                });
            })
        }
    };

    if props.resources.is_empty() {
        return html! {
            <div class="search-results">
                {"(no results)"}
            </div>
        };
    }

    html! {
        <ul class={"search-results"}>
            { props.resources.iter().filter_map(|rid| {
                if let Some(container) = graph_state.graph.get(&rid).as_ref() {
                    let mut class = classes!("search-result", "container", "clickable");
                    if canvas_state.selected.contains(rid) {
                        class.push("selected");
                    }

                    Some(html! {
                        <li {class}
                            data-id={rid.clone()}
                            onclick={toggle_select_container(rid.clone())}
                        >
                            <span class="icon">
                                <Icon
                                    icon_id={IconId::FontAwesomeRegularFolder}
                                    width={ICON_SIZE.to_string()} height={ICON_SIZE.to_string()}
                                />
                            </span>
                            <span class="name">
                                { &container.properties.name }
                            </span>
                        </li>
                    })
                } else if let Some(container) = graph_state.asset_map.get(&rid).as_ref() {
                    let container = graph_state.graph.get(container).unwrap();
                    let asset = container.assets.get(&rid).unwrap();
                    let mut class = classes!("search-result", "asset", "clickable");
                    if canvas_state.selected.contains(rid) {
                        class.push("selected");
                    }

                    Some(html! {
                        <li {class}
                            data-id={rid.clone()}
                            onclick={toggle_select_asset(rid.clone())}
                        >
                            <span class="icon">
                                <Icon
                                    icon_id={ui_common::asset::asset_icon_id(asset)}
                                    width={ICON_SIZE.to_string()} height={ICON_SIZE.to_string()}
                                    style={ui_common::asset::asset_icon_color(asset)}
                                />
                            </span>
                            <span class="name">
                                { ui_common::asset::asset_display_name(asset) }
                            </span>
                        </li>
                    })
                } else {
                    tracing::error!("resource {rid:?} not found in graph");
                    None
                }
            }).collect::<Vec<_>>() }
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
