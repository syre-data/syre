use serde::Serialize;
use std::{io, path::PathBuf};
use syre_desktop_lib::command::error::IoErrorKind;

pub async fn pick_folder(title: impl Into<String>) -> Option<PathBuf> {
    #[derive(Serialize)]
    struct Args {
        title: String,
    }

    tauri_sys::core::invoke(
        "pick_folder",
        Args {
            title: title.into(),
        },
    )
    .await
}

/// Open a folder picker dialog starting from the given directory.
pub async fn pick_folder_with_location(
    title: impl Into<String>,
    dir: impl Into<PathBuf>,
) -> Option<PathBuf> {
    #[derive(Serialize)]
    struct Args {
        title: String,
        dir: PathBuf,
    }

    tauri_sys::core::invoke(
        "pick_folder_with_location",
        Args {
            title: title.into(),
            dir: dir.into(),
        },
    )
    .await
}

/// Open the file or folder at the path with the default program.
pub async fn open_file(path: impl Into<PathBuf>) -> Result<(), io::ErrorKind> {
    #[derive(Serialize)]
    struct Args {
        path: PathBuf,
    }

    tauri_sys::core::invoke_result::<(), IoErrorKind>("open_file", Args { path: path.into() })
        .await
        .map_err(|err| err.into())
}
