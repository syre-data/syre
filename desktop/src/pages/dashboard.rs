use crate::{
    commands::fs::{pick_folder, pick_folder_with_location},
    components::{message::Builder as Message, ModalDialog, TruncateLeft},
    types,
};
use futures::stream::StreamExt;
use leptos::*;
use leptos_router::*;
use serde::Serialize;
use std::{path::PathBuf, rc::Rc, str::FromStr};
use syre_core::{project::Project, system::User, types::ResourceId};
use syre_desktop_lib as lib;
use syre_local as local;
use syre_local_database as db;
use tauri_sys::{core::Channel, menu};
use web_sys::{MouseEvent, SubmitEvent};

/// Context menu for containers that are `Ok`.
#[derive(derive_more::Deref, Clone)]
struct ContextMenuProjectOk(Rc<menu::Menu>);
impl ContextMenuProjectOk {
    pub fn new(menu: Rc<menu::Menu>) -> Self {
        Self(menu)
    }
}

/// Active project for the project context menu.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct ContextMenuActiveProject(PathBuf);

#[component]
pub fn Dashboard() -> impl IntoView {
    let user = expect_context::<User>();
    let messages = expect_context::<types::Messages>();
    let context_menu_active_project_ok = create_rw_signal::<Option<ContextMenuActiveProject>>(None);
    provide_context(context_menu_active_project_ok.clone());

    let projects = create_resource(|| (), {
        let user = user.rid().clone();
        move |_| {
            let user = user.clone();
            async move { fetch_user_projects(user).await }
        }
    });

    let context_menu_project_ok = create_local_resource(|| (), {
        let messages = messages.clone();
        move |_| {
            let messages = messages.clone();
            async move {
                let mut project_remove = tauri_sys::menu::item::MenuItemOptions::new("Remove");
                project_remove.set_id("dashboard:project-remove");

                let (menu, mut listeners) = menu::Menu::with_id_and_items(
                    "dashboard:project-ok-context_menu",
                    vec![project_remove.into()],
                )
                .await;

                spawn_local({
                    // pop from end to beginning
                    let project_remove = listeners.pop().unwrap().unwrap();
                    handle_context_menu_project_ok_events(
                        messages,
                        context_menu_active_project_ok.read_only(),
                        project_remove,
                    )
                });

                Rc::new(menu)
            }
        }
    });

    view! {
        <Suspense fallback=Loading>
            {move || {
                let context_menu_project_ok = context_menu_project_ok.get()?;
                let projects = projects.get()?;
                Some(view! { <DashboardView projects context_menu_project_ok /> })
            }}
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="text-center pt-4">"Loading projects..."</div> }
}

#[component]
fn DashboardView(
    projects: Vec<(PathBuf, db::state::ProjectData)>,
    context_menu_project_ok: Rc<menu::Menu>,
) -> impl IntoView {
    provide_context(ContextMenuProjectOk::new(context_menu_project_ok));
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
    let context_menu = expect_context::<ContextMenuProjectOk>();
    let context_menu_active_project =
        expect_context::<RwSignal<Option<ContextMenuActiveProject>>>();

    let path_str = {
        let path = path.clone();
        move || path.to_string_lossy().to_string()
    };

    let contextmenu = {
        let path = path.clone();
        move |e: MouseEvent| {
            e.prevent_default();

            context_menu_active_project.update(|active_project| {
                let _ = active_project.insert(path.clone().into());
            });

            let menu = context_menu.clone();
            spawn_local(async move {
                menu.popup().await.unwrap();
            });
        }
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
        });
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
        <div class="px-4 py-2 rounded border border-black bg-white dark:bg-secondary-800 dark:border-secondary-400">
            <div class="text-center text-2xl pb-2 dark:text-white">"Create a new project"</div>
            <form on:submit=create_project>
                <div class="pb-4">
                    <div class="flex gap-2">
                        <input
                            name="path"
                            prop:value=move || {
                                path.with(|path| match path {
                                    None => "".to_string(),
                                    Some(path) => path.to_string_lossy().to_string(),
                                })
                            }
                            class="grow"
                            readonly
                        />

                        <button class="btn btn-secondary" type="button" on:mousedown=select_path>
                            "Change"
                        </button>
                    </div>
                </div>
                <div class="flex gap-2 justify-center">
                    <button
                        disabled=move || path.with(|path| path.is_none())
                        class="btn btn-primary"
                    >
                        "Create"
                    </button>
                    <button type="button" on:mousedown=close class="btn btn-secondary">
                        "Cancel"
                    </button>
                </div>
                <div>{error}</div>
            </form>
        </div>
    }
}

#[component]
fn InitializeProject(
    children: Children,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] title: MaybeProp<String>,
) -> impl IntoView {
    let user = expect_context::<User>();
    let messages = expect_context::<types::Messages>();
    let initialize_project_action = create_action({
        let user = user.rid().clone();
        let messages = messages.clone();
        move |_| {
            let user = user.clone();
            let messages = messages.clone();
            async move {
                if let Some(path) = pick_folder("Initialize an existing directory").await {
                    if let Err(err) = initialize_project(user, path).await {
                        let mut msg = Message::error("Could not initialize project");
                        msg.body(format!("{err:?}"));
                        messages.update(|messages| messages.push(msg.build()));
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

async fn handle_context_menu_project_ok_events(
    messages: types::Messages,
    context_menu_active_project: ReadSignal<Option<ContextMenuActiveProject>>,
    project_remove: Channel<String>,
) {
    let mut project_remove = project_remove.fuse();
    loop {
        futures::select! {
            event = project_remove.next() => match event {
                None => continue,
                Some(_id) => {
                    let project = context_menu_active_project.get_untracked().unwrap();
                    if let Err(err) =  remove_project((*project).clone()).await {
                        messages.update(|messages|{
                            let mut msg = Message::error("Could not remove project.");
                            msg.body(format!("{err:?}"));
                            messages.push(msg.build());
                        });
                    }
                }
            },
        }
    }
}

async fn remove_project(project: PathBuf) -> Result<(), local::error::IoSerde> {
    #[derive(Serialize)]
    struct Args {
        project: PathBuf,
    }

    tauri_sys::core::invoke_result("deregister_project", Args { project }).await
}
