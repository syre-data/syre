use crate::{
    commands::fs::{pick_folder, pick_folder_with_location},
    components::ModalDialog,
    types::MouseButton,
};
use futures::stream::StreamExt;
use leptos::*;
use leptos_router::*;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::{project::Project, system::User, types::ResourceId};
use syre_desktop_lib as lib;
use syre_local_database as db;
use web_sys::{MouseEvent, SubmitEvent};

#[component]
pub fn Dashboard() -> impl IntoView {
    let user = expect_context::<User>();
    let projects = create_resource(|| (), {
        let user = user.rid().clone();
        move |_| {
            let user = user.clone();
            async move { fetch_user_projects(user).await }
        }
    });
    view! {
        <Suspense fallback=Loading>
            {move || projects.map(|projects| view! { <DashboardView projects=projects.clone()/> })}

        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div>"Loading projects..."</div> }
}

#[component]
fn DashboardView(projects: Vec<(PathBuf, db::state::ProjectData)>) -> impl IntoView {
    let (projects, set_projects) = create_signal(
        projects
            .into_iter()
            .map(|project| RwSignal::new(project))
            .collect::<Vec<_>>(),
    );

    let create_project_path = RwSignal::new(None);
    let create_project_ref = NodeRef::<html::Dialog>::new();

    spawn_local(async move {
        let mut listener =
            tauri_sys::event::listen::<Vec<lib::Event>>(lib::event::topic::PROJECT_MANIFEST)
                .await
                .unwrap();

        while let Some(events) = listener.next().await {
            for event in events.payload {
                let lib::EventKind::ProjectManifest(update) = event.kind() else {
                    panic!("invalid event kind");
                };

                match update {
                    lib::event::ProjectManifest::Added(states) => {
                        set_projects.update(|projects| {
                            projects.extend(
                                states
                                    .iter()
                                    .map(|state| RwSignal::new(state.clone()))
                                    .collect::<Vec<_>>(),
                            )
                        });
                    }
                    lib::event::ProjectManifest::Removed(removed) => {
                        set_projects.update(|projects| {
                            projects.retain(|project| {
                                project.with_untracked(|(path, _)| !removed.contains(path))
                            });
                        });
                    }
                    lib::event::ProjectManifest::Corrupted => todo!(),
                    lib::event::ProjectManifest::Repaired => todo!(),
                }
            }
        }
    });

    Effect::new(move |_| {
        if create_project_path.with(|path| path.is_none()) {
            let dialog = create_project_ref.get().unwrap();
            dialog.close();
        }
    });

    let show_create_project_dialog = move |e: MouseEvent| {
        spawn_local(async move {
            if let Some(p) = pick_folder("Create a new project").await {
                create_project_path.update(|path| {
                    let _ = path.insert(p);
                });
                let dialog = create_project_ref.get_untracked().unwrap();
                dialog.show_modal().unwrap();
            }
        })
    };

    view! {
        <div>
            <div>
                <button on:mousedown=show_create_project_dialog>"New project"</button>
            </div>

            <ProjectDeck projects/>

            <ModalDialog node_ref=create_project_ref>
                <CreateProject path=create_project_path/>
            </ModalDialog>
        </div>
    }
}

#[component]
fn ProjectDeck(
    projects: ReadSignal<Vec<RwSignal<(PathBuf, db::state::ProjectData)>>>,
) -> impl IntoView {
    view! {
        <div class="flex">
            <For
                each=projects
                key=|state| {
                    state
                        .with(|(path, project)| {
                            if let db::state::DataResource::Ok(properties) = project.properties() {
                                properties.rid().to_string()
                            } else {
                                path.to_string_lossy().to_string()
                            }
                        })
                }

                let:project
            >
                <ProjectCard project=project.read_only()/>
            </For>
        </div>
    }
}

#[component]
fn ProjectCard(project: ReadSignal<(PathBuf, db::state::ProjectData)>) -> impl IntoView {
    move || {
        project.with(|(path, project)| {
            if let db::state::DataResource::Ok(project) = project.properties() {
                view! { <ProjectCardOk project=project.clone() path=path.clone()/> }
            } else {
                view! { <ProjectCardErr path=path.clone()/> }
            }
        })
    }
}

#[component]
fn ProjectCardOk(project: Project, path: PathBuf) -> impl IntoView {
    let navigate = use_navigate();
    let goto_project = {
        let navigate = navigate.clone();
        let project = project.rid().to_string();
        move |e: MouseEvent| {
            if e.button() == MouseButton::Primary as i16 {
                navigate(&project, Default::default());
            }
        }
    };

    view! {
        <A href={
            let project = project.rid().clone();
            move || project.to_string()
        }>
            <div class="border-2 rounded border">
                <h3>{project.name.clone()}</h3>
                <div>{project.description.clone()}</div>
                <div>{path.to_string_lossy().to_string()}</div>
            </div>
        </A>
    }
}

#[component]
fn ProjectCardErr(path: PathBuf) -> impl IntoView {
    view! {
        <div>{path.to_string_lossy().to_string()}</div>
        <div>"is broken"</div>
    }
}

#[component]
fn CreateProject(path: RwSignal<Option<PathBuf>>) -> impl IntoView {
    let user = expect_context::<User>();
    let (error, set_error) = create_signal(None);
    let create_project = {
        move |e: SubmitEvent| {
            e.prevent_default();

            let user = user.rid().clone();
            match path() {
                None => {
                    set_error(Some("Path is required."));
                }
                Some(p) => {
                    spawn_local(async move {
                        match create_project(user, p).await {
                            Ok(_project) => {
                                path.update(|path| {
                                    path.take();
                                });
                            }
                            Err(err) => {
                                tracing::error!(?err);
                                set_error(Some("Could not create project."));
                            }
                        }
                    });
                }
            }
        }
    };

    let select_path = move |_| {
        spawn_local(async move {
            let init_dir = path.with(|path| match path {
                None => PathBuf::new(),
                Some(path) => path
                    .parent()
                    .map(|path| path.to_path_buf())
                    .unwrap_or(PathBuf::new()),
            });

            if let Some(p) = pick_folder_with_location("Create a new project", init_dir).await {
                path.update(|path| {
                    let _ = path.insert(p);
                });
            }
        });
    };

    let close = move |_| {
        path.update(|path| {
            path.take();
        });
    };

    view! {
        <form on:submit=create_project>
            <div>
                <input
                    name="path"
                    prop:value=move || {
                        path.with(|path| match path {
                            None => "".to_string(),
                            Some(path) => path.to_string_lossy().to_string(),
                        })
                    }

                    readonly
                />

                <button type="button" on:mousedown=select_path>
                    "Change"
                </button>
            </div>
            <div>
                <button disabled=move || path.with(|path| path.is_none())>"Create"</button>
                <button type="button" on:mousedown=close>
                    "Cancel"
                </button>
            </div>
            <div>{error}</div>
        </form>
    }
}

async fn fetch_user_projects(user: ResourceId) -> Vec<(PathBuf, db::state::ProjectData)> {
    tauri_sys::core::invoke("user_projects", UserProjectsArgs { user }).await
}

#[derive(Serialize)]
struct UserProjectsArgs {
    user: ResourceId,
}

async fn create_project(user: ResourceId, path: PathBuf) -> syre_local::Result<Project> {
    tauri_sys::core::invoke_result("create_project", CreateProjectArgs { user, path }).await
}

#[derive(Serialize)]
struct CreateProjectArgs {
    user: ResourceId,
    path: PathBuf,
}
