use super::Canvas;
use crate::invoke::invoke;
use leptos::*;
use leptos_router::use_params_map;
use serde::Serialize;
use std::str::FromStr;
use syre_core::types::ResourceId;
use syre_local_database as db;

#[component]
pub fn Workspace() -> impl IntoView {
    let params = use_params_map();
    let id =
        move || params.with(|params| ResourceId::from_str(&params.get("id").unwrap()).unwrap());
    let graph = create_resource(id, |id| async move { fetch_project_graph(id).await });

    view! {
        <Suspense fallback=Loading>
            {move || {
                graph
                    .get()
                    .map(|graph| {
                        graph
                            .map(|graph| {
                                view! {
                                    {if let db::state::FolderResource::Present(graph) = graph {
                                        view! { <WorkspaceView/> }
                                    } else {
                                        view! { <NoGraph/> }
                                    }}
                                }
                            })
                            .or_else(|| Some(view! { <NoProject/> }))
                    })
            }}

        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div>"Loading..."</div> }
}

#[component]
fn NoGraph() -> impl IntoView {
    view! { <div>"Data graph does not exist."</div> }
}

#[component]
fn NoProject() -> impl IntoView {
    view! { <div>"Project state was not found."</div> }
}

fn WorkspaceView() -> impl IntoView {
    view! {
        <div class="h-1/8 border-b-4">"tab bar"</div>
        <div class="h-1/6 border-b-4">"app bar"</div>
        <div class="flex">
            <div class="w-1/6 border-r-4">"left"</div>
            <div class="flex-grow">
                <Canvas/>
            </div>
            <div class="w-1/6 border-l-4">"right"</div>
        </div>
    }
}

async fn fetch_project_graph(
    project: ResourceId,
) -> Option<db::state::FolderResource<db::state::Graph>> {
    invoke("project_graph", ProjectGraphArgs { project }).await
}

#[derive(Serialize)]
struct ProjectGraphArgs {
    project: ResourceId,
}
