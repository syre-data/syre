use super::super::settings::{app::Settings as AppSettings, user::Settings as UserSettings};
use leptos::{either::either, ev::MouseEvent, prelude::*};
use leptos_icons::*;
use reactive_stores::Store;
use serde::Serialize;
use std::path::PathBuf;
use syre_desktop_lib as lib;
use syre_desktop_ui_lib as ui_lib;

#[derive(Clone, Copy)]
enum ActiveView {
    User,
    Project,
    App,
}

impl Default for ActiveView {
    fn default() -> Self {
        Self::User
    }
}

#[component]
pub fn Settings(
    /// Called when the user requests to close the page.
    #[prop(into)]
    onclose: Callback<()>,
) -> impl IntoView {
    let active_view = RwSignal::new(ActiveView::default());
    let trigger_close = move |e: MouseEvent| {
        if e.button() == ui_lib::types::MouseButton::Primary {
            onclose.run(());
        }
    };

    view! {
        <div class="flex relative bg-white dark:bg-secondary-800 dark:text-white h-full w-full">
            <Nav active_view />
            <div class="grow h-full">
                <SettingsView active_view=active_view.read_only() />
            </div>
            <div class="absolute top-2 right-2">
                <button
                    on:mousedown=trigger_close
                    type="button"
                    class="rounded-sm hover:bg-secondary-100 dark:hover:bg-secondary-700 cursor-pointer"
                    title="Close settings."
                >
                    <Icon icon=ui_lib::icon::Close />
                </button>
            </div>
        </div>
    }
}

#[component]
fn Nav(active_view: RwSignal<ActiveView>) -> impl IntoView {
    view! {
        <nav class="flex flex-col h-full bg-secondary-100 dark:bg-secondary-900">
            <ul class="grow pt-4">
                <li
                    class=(
                        ["bg-white", "dark:bg-secondary-800"],
                        move || matches!(active_view(), ActiveView::User),
                    )
                    class="px-2 border-b"
                    title="User settings"
                >
                    <button
                        type="button"
                        on:mousedown=move |_| active_view.set(ActiveView::User)
                        class="text-2xl p-2 cursor-pointer"
                    >
                        <Icon icon=ui_lib::icon::User />
                    </button>
                </li>
                <li
                    class=(
                        ["bg-white", "dark:bg-secondary-800"],
                        move || matches!(active_view(), ActiveView::Project),
                    )
                    class="px-2 border-b"
                    title="Project settings"
                >
                    <button
                        type="button"
                        on:mousedown=move |_| active_view.set(ActiveView::Project)
                        class="text-2xl p-2 cursor-pointer"
                    >
                        <Icon icon=icondata::LuNetwork />
                    </button>
                </li>
            </ul>
            <ul class="pb-4">
                <li
                    class=(
                        ["bg-white", "dark:bg-secondary-800"],
                        move || matches!(active_view(), ActiveView::App),
                    )
                    class="px-2 border-t"
                    title="App settings"
                >
                    <button
                        type="button"
                        on:mousedown=move |_| active_view.set(ActiveView::App)
                        class="text-2xl p-2 cursor-pointer"
                    >
                        <Icon icon=ui_lib::icon::Settings />
                    </button>
                </li>
            </ul>
        </nav>
    }
}

#[component]
fn SettingsView(active_view: ReadSignal<ActiveView>) -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let user_settings = expect_context::<Store<ui_lib::state::settings::User>>();
    let project_settings = expect_context::<Store<ui_lib::state::settings::Project>>();
    let user_settings_resource = LocalResource::new(fetch_user_settings);
    let project_settings_resource =
        LocalResource::new(move || fetch_project_settings(project.path().get_untracked()));

    view! {
        <Suspense fallback=Loading>
            {move || Suspend::new(async move {
                if let Some(settings) = user_settings_resource.await {
                    user_settings.set(settings.into());
                }
                if let Some(settings) = project_settings_resource.await {
                    project_settings.set(settings.into());
                }
                either!(
                    active_view(),
                    ActiveView::Project => project::Settings,
                    ActiveView::User => UserSettings,
                    ActiveView::App => AppSettings,
                )
            })}
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="pt-2 text-center">"Loading settings"</div> }
}

async fn fetch_user_settings() -> Option<lib::settings::User> {
    tauri_sys::core::invoke("user_settings", ()).await
}

async fn fetch_project_settings(project: PathBuf) -> Option<lib::settings::Project> {
    #[derive(Serialize)]
    struct ProjectArgs {
        project: PathBuf,
    }

    tauri_sys::core::invoke("project_settings", ProjectArgs { project }).await
}

mod project {
    use leptos::prelude::*;
    use reactive_stores::Store;
    use syre_desktop_lib as lib;
    use syre_desktop_ui_lib as ui_lib;

    #[derive(derive_more::Deref, Clone, Copy)]
    pub struct InputDebounce(Signal<f64>);

    #[component]
    pub fn Settings() -> impl IntoView {
        let user_settings = expect_context::<Store<ui_lib::state::settings::User>>();

        provide_context(InputDebounce(Signal::derive(move || {
            let debounce = match &user_settings.read().desktop {
                Ok(settings) => settings.input_debounce_ms,
                Err(_) => lib::settings::user::Desktop::default().input_debounce_ms,
            };

            debounce as f64
        })));

        view! {
            <div class="relative bg-white dark:bg-secondary-800 dark:text-white h-full w-full">
                <h1 class="text-lg font-primary pt-2 pb-4 px-2">"Project settings"</h1>
                <div class="px-2">
                    <h2 class="text-md font-primary pb-2">"Desktop"</h2>
                    <desktop::Settings />
                </div>
                <div class="px-2">
                    <h2 class="text-md font-primary pb-2">"Runner"</h2>
                    <runner::Settings />
                    <analysis::Settings />
                </div>
            </div>
        }
    }

    mod desktop {
        use super::InputDebounce;
        use leptos::{ev::Event, prelude::*, task::spawn_local};
        use reactive_stores::Store;
        use serde::Serialize;
        use std::{io, path::PathBuf};
        use syre_desktop_lib as lib;
        use syre_desktop_ui_lib::{self as ui_lib, state::settings::project::SettingsStoreFields};
        use syre_local::error::IoSerde;

        #[component]
        pub fn Settings() -> impl IntoView {
            let project = expect_context::<ui_lib::state::Project>();
            let project_settings = expect_context::<Store<ui_lib::state::settings::Project>>();
            let settings = project_settings.desktop();
            let input_debounce = expect_context::<InputDebounce>();
            let messages = expect_context::<ui_lib::message::Messages>();

            let (asset_drag_drop_kind, set_asset_drag_drop_kind) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.asset_drag_drop_kind.clone())
                    .ok()
                    .flatten(),
            );
            let asset_drag_drop_kind: Signal<Option<String>> =
                leptos_use::signal_debounced(asset_drag_drop_kind, *input_debounce);

            let _ = {
                let project = project.path().get_untracked();
                Effect::watch(
                    move || (asset_drag_drop_kind.get(),),
                    move |(asset_drag_drop_kind,), _, _| {
                        let update = match &settings.get_untracked() {
                            Ok(settings) => Ok(settings.clone()),
                            Err(err) if matches!(err, IoSerde::Io(io::ErrorKind::NotFound)) => {
                                Ok(lib::settings::project::Desktop::default().into())
                            }
                            Err(err) => Err(err.clone()),
                        };

                        let mut update = match update {
                            Ok(update) => update,
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                                return;
                            }
                        };

                        update.asset_drag_drop_kind = asset_drag_drop_kind.clone();
                        project_settings.update(|settings| {
                            settings.desktop = Ok(update.clone());
                        });
                        let project = project.clone();
                        spawn_local(async move {
                            if let Err(err) = update_settings(project, update.into()).await {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }
                        });
                    },
                    false,
                )
            };

            view! {
                <form on:submit=move |e| e.prevent_default()>
                    <div class="pb-2">
                        <AssetDragDropKind
                            value=asset_drag_drop_kind
                            set_value=set_asset_drag_drop_kind
                        />
                    </div>
                </form>
            }
        }

        #[component]
        fn AssetDragDropKind(
            value: Signal<Option<String>>,
            set_value: WriteSignal<Option<String>>,
        ) -> impl IntoView {
            let update_kind = move |e: Event| {
                let value = event_target_value(&e);
                let value = value.trim();
                if value.is_empty() {
                    let _ = set_value.write().take();
                } else {
                    let _ = set_value.write().insert(value.to_string());
                }
            };

            view! {
                <label
                    title="Type assigned to assets on drag-drop."
                    class="flex gap-2 items-center"
                >
                    <span class="text-nowrap">"Asset drag drop type"</span>
                    <div class="grow flex gap-2 items-center">
                        <input
                            prop:value=value
                            on:input=update_kind
                            class="input-simple grow"
                            placeholder="Type"
                        />
                    </div>
                </label>
            }
        }

        async fn update_settings(
            project: PathBuf,
            update: lib::settings::project::Desktop,
        ) -> Result<(), IoSerde> {
            #[derive(Serialize)]
            struct Args {
                project: PathBuf,
                update: lib::settings::project::Desktop,
            }

            tauri_sys::core::invoke_result(
                "project_settings_desktop_update",
                Args { project, update },
            )
            .await
        }
    }

    mod runner {
        use super::InputDebounce;
        use leptos::{
            either::Either,
            ev::{Event, MouseEvent},
            html,
            prelude::*,
            task::spawn_local,
        };
        use leptos_icons::*;
        use reactive_stores::Store;
        use serde::Serialize;
        use std::{io, num::NonZeroUsize, path::PathBuf};
        use syre_desktop_lib as lib;
        use syre_desktop_ui_lib::{self as ui_lib, state::settings::project::SettingsStoreFields};
        use syre_local::error::IoSerde;

        #[component]
        pub fn Settings() -> impl IntoView {
            let project = expect_context::<ui_lib::state::Project>();
            let project_settings = expect_context::<Store<ui_lib::state::settings::Project>>();
            let settings = project_settings.runner();
            let input_debounce = expect_context::<InputDebounce>();
            let messages = expect_context::<ui_lib::message::Messages>();

            let (python_path, set_python_path) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.python_path.clone())
                    .ok()
                    .flatten(),
            );
            let python_path: Signal<Option<PathBuf>> =
                leptos_use::signal_debounced(python_path, *input_debounce);

            let (r_path, set_r_path) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.r_path.clone())
                    .ok()
                    .flatten(),
            );
            let r_path: Signal<Option<PathBuf>> =
                leptos_use::signal_debounced(r_path, *input_debounce);

            let (max_tasks, set_max_tasks) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.max_tasks.clone())
                    .ok()
                    .flatten(),
            );
            let max_tasks: Signal<Option<NonZeroUsize>> =
                leptos_use::signal_debounced(max_tasks, *input_debounce);

            let (continue_on_error, set_continue_on_error) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.continue_on_error.clone())
                    .unwrap_or(None),
            );
            let continue_on_error =
                leptos_use::signal_debounced(continue_on_error, *input_debounce);

            let _ = {
                let project = project.path().get_untracked();
                Effect::watch(
                    move || {
                        (
                            python_path.get(),
                            r_path.get(),
                            max_tasks.get(),
                            continue_on_error.get(),
                        )
                    },
                    move |(python_path, r_path, max_tasks, continue_on_error), _, _| {
                        let update = match &settings.get_untracked() {
                            Ok(settings) => Ok(settings.clone()),
                            Err(err) if matches!(err, IoSerde::Io(io::ErrorKind::NotFound)) => {
                                Ok(lib::settings::project::Runner::default().into())
                            }
                            Err(err) => Err(err.clone()),
                        };

                        let mut update = match update {
                            Ok(update) => update,
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                                return;
                            }
                        };

                        update.python_path = python_path.clone();
                        update.r_path = r_path.clone();
                        update.max_tasks = max_tasks.clone();
                        update.continue_on_error = *continue_on_error;
                        project_settings.update(|settings| {
                            settings.runner = Ok(update.clone());
                        });
                        let project = project.clone();
                        spawn_local(async move {
                            if let Err(err) = update_settings(project, update.into()).await {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }
                        });
                    },
                    false,
                )
            };

            view! {
                <form on:submit=move |e| e.prevent_default()>
                    <div class="pb-2">
                        <PythonPath value=python_path set_value=set_python_path />
                    </div>
                    <div class="pb-2">
                        <RPath value=r_path set_value=set_r_path />
                    </div>
                    <div class="pb-2">
                        <MaxTasks value=max_tasks set_value=set_max_tasks />
                    </div>
                    <div class="pb-2">
                        <ContinueOnError value=continue_on_error set_value=set_continue_on_error />
                    </div>

                </form>
            }
        }

        #[component]
        fn PythonPath(
            value: Signal<Option<PathBuf>>,
            set_value: WriteSignal<Option<PathBuf>>,
        ) -> impl IntoView {
            let update_path = move |e: Event| {
                let value = event_target_value(&e);
                let value = value.trim();
                if value.is_empty() {
                    set_value.update(|path| {
                        let _ = path.take();
                    });
                } else {
                    set_value.update(|path| {
                        let _ = path.insert(PathBuf::from(value));
                    });
                }
            };

            let select_path = move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                spawn_local(async move {
                    let init_dir = value.with_untracked(|path| match path {
                        None => PathBuf::new(),
                        Some(path) => path
                            .parent()
                            .map(|path| path.to_path_buf())
                            .unwrap_or(PathBuf::new()),
                    });

                    if let Some(p) =
                        ui_lib::commands::fs::pick_file_with_location("Python path", init_dir).await
                    {
                        set_value.update(|path| {
                            let _ = path.insert(p);
                        });
                    }
                });
            };

            view! {
                <label class="flex gap-2 items-center">
                    <span>
                        <Icon icon=icondata::FaPythonBrands />
                    </span>
                    <span class="text-nowrap">"Python path"</span>
                    <button
                        type="button"
                        on:mousedown=select_path
                        class="aspect-square p-1 rounded-xs border border-black dark:border-white"
                    >
                        <Icon icon=icondata::FaFolderOpenRegular />
                    </button>
                    <input
                        type="text"
                        prop:value=move || {
                            value
                                .with(|path| {
                                    path.as_ref()
                                        .map(|path| path.to_string_lossy().to_string())
                                        .unwrap_or("".to_string())
                                })
                        }
                        on:input=update_path
                        class="input-simple grow"
                        placeholder="Python executable path"
                    />
                </label>
            }
        }

        #[component]
        fn RPath(
            value: Signal<Option<PathBuf>>,
            set_value: WriteSignal<Option<PathBuf>>,
        ) -> impl IntoView {
            let update_path = move |e: Event| {
                let value = event_target_value(&e);
                let value = value.trim();
                if value.is_empty() {
                    set_value.update(|path| {
                        let _ = path.take();
                    });
                } else {
                    set_value.update(|path| {
                        let _ = path.insert(PathBuf::from(value));
                    });
                }
            };

            let select_path = move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                spawn_local(async move {
                    let init_dir = value.with_untracked(|path| match path {
                        None => PathBuf::new(),
                        Some(path) => path
                            .parent()
                            .map(|path| path.to_path_buf())
                            .unwrap_or(PathBuf::new()),
                    });

                    if let Some(p) =
                        ui_lib::commands::fs::pick_file_with_location("R path", init_dir).await
                    {
                        set_value.update(|path| {
                            let _ = path.insert(p);
                        });
                    }
                });
            };

            view! {
                <label class="flex gap-2 items-center">
                    <span>
                        <Icon icon=icondata::FaRProjectBrands />
                    </span>
                    <span class="text-nowrap">"R path"</span>
                    <button
                        type="button"
                        on:mousedown=select_path
                        class="aspect-square p-1 rounded-xs border border-black dark:border-white"
                    >
                        <Icon icon=icondata::FaFolderOpenRegular />
                    </button>
                    <input
                        type="text"
                        prop:value=move || {
                            value
                                .with(|path| {
                                    path.as_ref()
                                        .map(|path| path.to_string_lossy().to_string())
                                        .unwrap_or("".to_string())
                                })
                        }
                        on:input=update_path
                        class="input-simple grow"
                        placeholder="R executable path"
                    />
                </label>
            }
        }

        #[component]
        fn MaxTasks(
            value: Signal<Option<NonZeroUsize>>,
            set_value: WriteSignal<Option<NonZeroUsize>>,
        ) -> impl IntoView {
            let (error, set_error) = signal(None);
            let update_tasks = move |e: Event| {
                set_error(None);
                let value = event_target_value(&e);
                let value = value.trim();
                if value.is_empty() {
                    set_value.update(|tasks| {
                        let _ = tasks.take();
                    });
                } else {
                    match value.parse::<NonZeroUsize>() {
                        Ok(value) => set_value.update(|tasks| {
                            let _ = tasks.insert(value);
                        }),
                        Err(_) => set_error.update(|error| {
                            let _ = error.insert("Invalid number");
                        }),
                    }
                }
            };

            view! {
                <label
                    title="Maximum number of tasks to run in parallel during analysis."
                    class="flex gap-2 items-center"
                >
                    <span class="text-nowrap">"Max tasks"</span>
                    <div class="grow flex gap-2 items-center">
                        <input
                            type="number"
                            prop:value=move || value.get().map(|value| value.to_string())
                            on:input=update_tasks
                            class=(
                                ["border-syre-red-600", "focus:ring-syre-red-600"],
                                move || error.read().is_some(),
                            )
                            class="input-simple grow"
                            placeholder="Max tasks"
                            min="1"
                        />
                        {move || {
                            if let Some(error) = error.get() {
                                Either::Left(
                                    view! {
                                        <small class="text-nowrap text-syre-red-600">{error}</small>
                                    },
                                )
                            } else {
                                Either::Right(())
                            }
                        }}
                    </div>
                </label>
            }
        }

        #[component]
        fn ContinueOnError(
            value: Signal<Option<bool>>,
            set_value: WriteSignal<Option<bool>>,
        ) -> impl IntoView {
            let node_ref = NodeRef::<html::Input>::new();
            node_ref.on_load(move |elm| {
                match *value.read() {
                    Some(true) => {
                        elm.set_checked(true);
                        elm.set_indeterminate(false);
                    }
                    Some(false) => {
                        elm.set_checked(false);
                        elm.set_indeterminate(false);
                    }
                    None => {
                        // elm.set_checked(false);
                        elm.set_indeterminate(true);
                    }
                }
            });

            Effect::new(move || {
                let Some(elm) = node_ref.get() else {
                    return;
                };

                match *value.read() {
                    Some(true) => {
                        elm.set_checked(true);
                        elm.set_indeterminate(false);
                    }
                    Some(false) => {
                        elm.set_checked(false);
                        elm.set_indeterminate(false);
                    }
                    None => {
                        // elm.set_checked(false);
                        elm.set_indeterminate(true);
                    }
                }
            });

            // TODO: Should be set on `<input>` but was recieving error, so moved to `<label> as workaround.
            let on_input = move |_| {
                let Some(elm) = node_ref.get() else {
                    return;
                };

                match *value.read_untracked() {
                    None => {
                        elm.set_checked(true);
                        elm.set_indeterminate(false);
                        set_value(Some(true));
                    }
                    Some(true) => {
                        elm.set_checked(false);
                        elm.set_indeterminate(false);
                        set_value(Some(false));
                    }
                    Some(false) => {
                        elm.set_checked(false);
                        elm.set_indeterminate(true);
                        set_value(None);
                    }
                }
            };

            view! {
                <label on:click=on_input class="flex gap-2 items-center cursor-pointer">
                    <input node_ref=node_ref type="checkbox" class="input-simple" />
                    "Continue analysis on error."
                </label>
            }
        }

        async fn update_settings(
            project: PathBuf,
            update: lib::settings::project::Runner,
        ) -> Result<(), lib::command::error::IoErrorKind> {
            #[derive(Serialize)]
            struct Args {
                project: PathBuf,
                update: lib::settings::project::Runner,
            }

            tauri_sys::core::invoke_result(
                "project_settings_runner_update",
                Args { project, update },
            )
            .await
        }
    }

    mod analysis {
        use super::InputDebounce;
        use leptos::{html, prelude::*, task::spawn_local};
        use reactive_stores::Store;
        use serde::Serialize;
        use std::{io, path::PathBuf};
        use syre_desktop_lib as lib;
        use syre_desktop_ui_lib::{self as ui_lib, state::settings::project::SettingsStoreFields};
        use syre_local::error::IoSerde;

        #[component]
        pub fn Settings() -> impl IntoView {
            let project = expect_context::<ui_lib::state::Project>();
            let project_settings = expect_context::<Store<ui_lib::state::settings::Project>>();
            let settings = project_settings.analysis();
            let input_debounce = expect_context::<InputDebounce>();
            let messages = expect_context::<ui_lib::message::Messages>();

            let (disable_analysis_after, set_disable_analysis_after) = signal(
                settings
                    .read_untracked()
                    .as_ref()
                    .map(|settings| settings.disable_analysis_after)
                    .unwrap_or(None),
            );
            let disable_analysis_after =
                leptos_use::signal_debounced(disable_analysis_after, *input_debounce);

            let _ = {
                let project = project.path().get_untracked();
                Effect::watch(
                    move || (disable_analysis_after.get(),),
                    move |(disable_analysis_after,), _, _| {
                        let update = match &settings.get_untracked() {
                            Ok(settings) => Ok(settings.clone()),
                            Err(err) if matches!(err, IoSerde::Io(io::ErrorKind::NotFound)) => {
                                Ok(lib::settings::project::Analysis::default().into())
                            }
                            Err(err) => Err(err.clone()),
                        };

                        let mut update = match update {
                            Ok(update) => update,
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                                return;
                            }
                        };

                        update.disable_analysis_after = *disable_analysis_after;
                        project_settings.update(|settings| {
                            settings.analysis = Ok(update.clone());
                        });
                        let project = project.clone();
                        spawn_local(async move {
                            if let Err(err) = update_settings(project, update.into()).await {
                                let msg =
                                    ui_lib::message::Builder::error("Could not update settings.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }
                        });
                    },
                    false,
                )
            };

            view! {
                <form on:submit=move |e| e.prevent_default()>
                    <div class="pb-2">
                        <DisableAnalysisAfter
                            value=disable_analysis_after
                            set_value=set_disable_analysis_after
                        />
                    </div>
                </form>
            }
        }

        #[component]
        fn DisableAnalysisAfter(
            value: Signal<Option<lib::settings::analysis::DisableAnalysisAfter>>,
            set_value: WriteSignal<Option<lib::settings::analysis::DisableAnalysisAfter>>,
        ) -> impl IntoView {
            use lib::settings::analysis::DisableAnalysisAfter;

            let node_ref = NodeRef::<html::Select>::new();
            let _ = Effect::watch(
                move || node_ref.get(),
                move |node_ref, _, _| {
                    let Some(input) = node_ref else {
                        return;
                    };

                    value.with_untracked(|value| {
                        let value = match value {
                            None => "",
                            Some(value) => diasble_analysis_after_to_str(value),
                        };
                        input.set_value(value);
                    });
                },
                true,
            );

            let on_input = move |e| {
                let value = event_target_value(&e);
                let value = if value.is_empty() {
                    None
                } else {
                    Some(disable_analysis_after_from_str(value.as_str()).unwrap())
                };

                set_value(value);
            };

            view! {
                <label title="Automatically disable analysis associations after they run.">
                    <span class="no-wrap pr-2">"Disable analyses after"</span>
                    <select node_ref=node_ref on:input=on_input class="input-compact">
                        <option value="">"-"</option>
                        <option value=diasble_analysis_after_to_str(
                            &DisableAnalysisAfter::False,
                        )>"Never"</option>
                        <option value=diasble_analysis_after_to_str(
                            &DisableAnalysisAfter::SuccessNoFlags,
                        )>"Success without flags"</option>
                        <option value=diasble_analysis_after_to_str(
                            &DisableAnalysisAfter::Success,
                        )>"Success"</option>
                        <option value=diasble_analysis_after_to_str(
                            &DisableAnalysisAfter::True,
                        )>"Always"</option>
                    </select>
                </label>
            }
        }

        fn diasble_analysis_after_to_str(
            value: &lib::settings::analysis::DisableAnalysisAfter,
        ) -> &'static str {
            use lib::settings::analysis::DisableAnalysisAfter;

            match value {
                DisableAnalysisAfter::False => "false",
                DisableAnalysisAfter::SuccessNoFlags => "success_no_flags",
                DisableAnalysisAfter::Success => "success",
                DisableAnalysisAfter::True => "true",
            }
        }

        fn disable_analysis_after_from_str(
            value: &str,
        ) -> Option<lib::settings::analysis::DisableAnalysisAfter> {
            use lib::settings::analysis::DisableAnalysisAfter;

            match value {
                "false" => Some(DisableAnalysisAfter::False),
                "success_no_flags" => Some(DisableAnalysisAfter::SuccessNoFlags),
                "success" => Some(DisableAnalysisAfter::Success),
                "true" => Some(DisableAnalysisAfter::True),
                _ => None,
            }
        }
        async fn update_settings(
            project: PathBuf,
            update: lib::settings::project::Analysis,
        ) -> Result<(), lib::command::error::IoErrorKind> {
            #[derive(Serialize)]
            struct Args {
                project: PathBuf,
                update: lib::settings::project::Analysis,
            }

            tauri_sys::core::invoke_result(
                "project_settings_analysis_update",
                Args { project, update },
            )
            .await
        }
    }
}
