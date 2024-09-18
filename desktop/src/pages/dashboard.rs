use crate::{
    commands::fs::{pick_folder, pick_folder_with_location},
    components::{message::Builder as Message, ModalDialog, TruncateLeft},
    types,
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
            {move || projects.map(|projects| view! { <DashboardView projects=projects.clone() /> })}
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="text-center pt-4">"Loading projects..."</div> }
}

#[component]
fn DashboardView(projects: Vec<(PathBuf, db::state::ProjectData)>) -> impl IntoView {
    let (projects, set_projects) = create_signal(
        projects
            .into_iter()
            .map(|project| RwSignal::new(project))
            .collect::<Vec<_>>(),
    );

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

    view! {
        <div class="p-4">
            <Show
                when=move || projects.with(|projects| !projects.is_empty())
                fallback=|| view! { <DashboardNoProjects /> }
            >
                <DashboardProjects projects />
            </Show>
        </div>
    }
}

#[component]
fn DashboardNoProjects() -> impl IntoView {
    view! {
        <div>
            <div class="pb-2 font-primary text-3xl">"Dashboard"</div>
            <div>
                <div class="text-xl text-center pb-2">"Create your first project"</div>
                <div class="flex gap-y-2 flex-col items-center">
                    <CreateProject
                        class="btn btn-primary w-1/2"
                        title="Create a new Syre project from scratch."
                    >
                        <strong>"New"</strong>
                    </CreateProject>

                    <InitializeProject
                        class="btn btn-secondary w-1/2"
                        title="Initialize an existing folder as a Syre project."
                    >
                        <strong>"Initialize"</strong>
                        " an existing directory"
                    </InitializeProject>

                    <ImportProject
                        class="btn btn-secondary w-1/2"
                        title="If you already have a Syre project, import it into your workspace."
                    >
                        <strong>"Import"</strong>
                        " an existing project"
                    </ImportProject>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DashboardProjects(
    projects: ReadSignal<Vec<RwSignal<(PathBuf, db::state::ProjectData)>>>,
) -> impl IntoView {
    view! {
        <div class="pb-4">
            <span class="font-primary text-3xl pr-4">"Dashboard"</span>
            <div class="inline-flex gap-x-2 align-bottom">
                <CreateProject class="btn btn-primary">"New"</CreateProject>
                <InitializeProject class="btn btn-secondary">"Initialize"</InitializeProject>
                <ImportProject class="btn btn-secondary">"Import"</ImportProject>
            </div>
        </div>

        <ProjectDeck projects />
    }
}

#[component]
fn ProjectDeck(
    projects: ReadSignal<Vec<RwSignal<(PathBuf, db::state::ProjectData)>>>,
) -> impl IntoView {
    view! {
        <div class="flex gap-4">
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
                <ProjectCard project=project.read_only() />
            </For>
        </div>
    }
}

#[component]
fn ProjectCard(project: ReadSignal<(PathBuf, db::state::ProjectData)>) -> impl IntoView {
    move || {
        project.with(|(path, project)| {
            if let db::state::DataResource::Ok(project) = project.properties() {
                view! { <ProjectCardOk project=project.clone() path=path.clone() /> }
            } else {
                view! { <ProjectCardErr path=path.clone() /> }
            }
        })
    }
}

#[component]
fn ProjectCardOk(project: Project, path: PathBuf) -> impl IntoView {
    let path_str = move || path.to_string_lossy().to_string();
    let contextmenu = move |e: MouseEvent| {
        tracing::debug!("ctx");
    };

    view! {
        <A
            href={
                let project = project.rid().clone();
                move || project.to_string()
            }
            on:contextmenu=contextmenu
            class="w-1/4 rounded border border-secondary-900 dark:bg-secondary-700 dark:border-secondary-50"
        >
            <div class="px-4 py-2">
                <h3 class="text-2xl font-primary">{project.name.clone()}</h3>
                <div>{project.description.clone()}</div>
                <div title=path_str.clone() class="text-sm">
                    <TruncateLeft>{path_str.clone()}</TruncateLeft>
                </div>
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
fn CreateProject(
    children: Children,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] title: MaybeProp<String>,
) -> impl IntoView {
    let create_project_path = RwSignal::new(None);
    let create_project_ref = NodeRef::<html::Dialog>::new();

    let show_create_project_dialog = move |e: MouseEvent| {
        if e.button() != types::MouseButton::Primary {
            return;
        }

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

    Effect::new(move |_| {
        if create_project_path.with(|path| path.is_none()) {
            let dialog = create_project_ref.get().unwrap();
            dialog.close();
        }
    });

    view! {
        <button on:mousedown=show_create_project_dialog class=class title=title>
            {children()}
        </button>

        <ModalDialog node_ref=create_project_ref>
            <CreateProjectDialog path=create_project_path />
        </ModalDialog>
    }
}

#[component]
fn CreateProjectDialog(path: RwSignal<Option<PathBuf>>) -> impl IntoView {
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

#[component]
fn InitializeProject(
    children: Children,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] title: MaybeProp<String>,
) -> impl IntoView {
    let user = expect_context::<User>();
    let initialize_project_action = create_action({
        let user = user.rid().clone();
        move |_| {
            let user = user.clone();
            async move {
                if let Some(path) = pick_folder("Initialize an existing directory").await {
                    if let Err(err) = initialize_project(user, path).await {
                        todo!("{err:?}");
                    }
                }
            }
        }
    });

    let trigger_initialize_project = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            initialize_project_action.dispatch(());
        }
    };

    view! {
        <button on:mousedown=trigger_initialize_project class=class title=title>
            {children()}
        </button>
    }
}

#[component]
fn ImportProject(
    children: Children,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] title: MaybeProp<String>,
) -> impl IntoView {
    let user = expect_context::<User>();
    let messages = expect_context::<types::Messages>();

    let import_project_action = create_action({
        let user = user.rid().clone();
        move |_| {
            let user = user.clone();
            let messages = messages.clone();
            async move {
                if let Some(path) = pick_folder("Import a project").await {
                    if let Err(err) = import_project(user, path).await {
                        let mut msg = Message::error("Could not import project");
                        msg.body(format!("{err:?}"));
                        messages.update(|messages| messages.push(msg.build()));
                    }
                }
            }
        }
    });

    let trigger_import_project = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            import_project_action.dispatch(());
        }
    };

    view! {
        <button on:mousedown=trigger_import_project class=class title=title>
            {children()}
        </button>
    }
}

async fn fetch_user_projects(user: ResourceId) -> Vec<(PathBuf, db::state::ProjectData)> {
    tauri_sys::core::invoke("user_projects", UserProjectsArgs { user }).await
}

#[derive(Serialize)]
struct UserProjectsArgs {
    user: ResourceId,
}

async fn create_project(
    user: ResourceId,
    path: PathBuf,
) -> Result<(), lib::command::project::error::Initialize> {
    #[derive(Serialize)]
    struct Args {
        user: ResourceId,
        path: PathBuf,
    }

    tauri_sys::core::invoke_result("create_project", Args { user, path }).await
}

async fn initialize_project(
    user: ResourceId,
    path: PathBuf,
) -> Result<(), lib::command::project::error::Initialize> {
    #[derive(Serialize)]
    struct Args {
        user: ResourceId,
        path: PathBuf,
    }

    tauri_sys::core::invoke_result("initialize_project", Args { user, path }).await
}

async fn import_project(
    user: ResourceId,
    path: PathBuf,
) -> Result<(), lib::command::project::error::Import> {
    #[derive(Serialize)]
    struct Args {
        user: ResourceId,
        path: PathBuf,
    }

    tauri_sys::core::invoke_result("import_project", Args { user, path }).await
}
