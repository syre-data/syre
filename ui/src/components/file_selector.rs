//! Retrieve a user selected file.
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct FileFilter<'a> {
    pub name: &'a str,
    pub extensions: &'a [&'a str],
}

#[derive(Clone, PartialEq)]
pub enum FileSelectorAction<'a> {
    PickFile,
    PickFileFiltered(FileFilter<'a>),
    PickFolder,
}

#[derive(Properties, PartialEq)]
pub struct FileSelectorProps {
    pub title: AttrValue,
    pub action: FileSelectorAction<'static>,

    #[prop_or_default]
    pub default_path: Option<PathBuf>,

    /// Open immedialely into a file selection.
    #[prop_or(true)]
    pub select_on_open: bool,

    /// Show the cancel button.
    #[prop_or(true)]
    pub show_cancel: bool,

    /// Called if the selection is canceled.
    /// Only relevant if `show_cancel` or `select_on_open` are `true`.
    #[prop_or_default]
    pub oncancel: Callback<()>,

    #[prop_or_default]
    pub onsuccess: Callback<PathBuf>,

    #[prop_or_default]
    pub onerror: Callback<String>,
}

/// Allow the user to select a file
#[function_component(FileSelector)]
pub fn file_selector(props: &FileSelectorProps) -> Html {
    let path: UseStateHandle<Option<PathBuf>> = use_state(|| None);

    let onsubmit = use_callback(
        (props.onsuccess.clone(), path.clone()),
        move |_: MouseEvent, (onsuccess, path)| {
            let Some(path) = path.as_ref() else {
                return;
            };

            onsuccess.emit(path.clone());
        },
    );

    let onchange = use_callback(
        (
            props.title.clone(),
            props.action.clone(),
            path.clone(),
            props.default_path.clone(),
            props.onerror.clone(),
        ),
        {
            move |_: MouseEvent, (title, action, path, default_path, onerror)| {
                let title = title.clone();
                let action = action.clone();
                let default_path = path.as_ref().cloned().or_else(|| default_path.clone());
                let path = path.setter();
                let onerror = onerror.clone();

                spawn_local(async move {
                    match get_user_path(title.as_str(), &action, default_path.as_ref()).await {
                        Ok(Some(user_path)) => {
                            path.set(Some(user_path));
                        }

                        Ok(None) => {}

                        Err(err) => {
                            onerror.emit(format!("{err:?}"));
                        }
                    }
                });
            }
        },
    );

    let oncancel = use_callback(props.oncancel.clone(), move |_: MouseEvent, oncancel| {
        oncancel.emit(());
    });

    // get initial location
    use_effect_with(
        (
            props.title.clone(),
            props.action.clone(),
            props.default_path.clone(),
            props.select_on_open.clone(),
            props.oncancel.clone(),
            props.onerror.clone(),
        ),
        {
            let path = path.setter();
            move |(title, action, default_path, select_on_open, oncancel, onerror)| {
                if *select_on_open {
                    let title = title.clone();
                    let action = action.clone();
                    let default_path = default_path.clone();
                    let oncancel = oncancel.clone();
                    let onerror = onerror.clone();

                    spawn_local(async move {
                        match get_user_path(title.as_str(), &action, default_path.as_ref()).await {
                            Ok(Some(user_path)) => path.set(Some(user_path)),
                            Ok(None) => oncancel.emit(()),
                            Err(err) => onerror.emit(format!("{err:?}")),
                        }
                    });
                }
            }
        },
    );

    html! {
        <div class={"thot-ui-file-selector"}>
            <div class={"path-control"}>
                <span class={"path"}>
                    if let Some(path) = path.as_ref() {
                        { path.to_str().expect("could not convert path to str") }
                    } else {
                        { &props.title }
                    }
                </span>

                <button onclick={onchange}>
                    if path.is_none() {
                        { "Set" }
                    } else {
                        { "Change" }
                    }
                </button>
            </div>
            <div class={"controls"}>
                if props.show_cancel {
                    <button onclick={oncancel}>
                        { "Cancel" }
                    </button>
                }

                <button
                    disabled={path.is_none()}
                    onclick={onsubmit}>

                    { "Ok" }
                </button>
            </div>
        </div>
    }
}

async fn get_user_path<'a>(
    title: &str,
    action: &FileSelectorAction<'a>,
    default_path: Option<&PathBuf>,
) -> Result<Option<PathBuf>, tauri_sys::Error> {
    let mut file_selector = FileDialogBuilder::new();
    file_selector.set_title(title);
    if let Some(default_path) = default_path {
        file_selector.set_default_path(default_path);
    }

    match action {
        FileSelectorAction::PickFile => file_selector.pick_file().await,
        FileSelectorAction::PickFileFiltered(filter) => {
            file_selector
                .add_filter(filter.name, filter.extensions)
                .pick_file()
                .await
        }
        FileSelectorAction::PickFolder => file_selector.pick_folder().await,
    }
}
