use crate::invoke::invoke;
use serde::Serialize;
use std::path::PathBuf;

pub async fn pick_folder(title: impl Into<String>) -> Option<PathBuf> {
    #[derive(Serialize)]
    struct PickFolderArgs {
        title: String,
    }

    invoke(
        "pick_folder",
        PickFolderArgs {
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
    struct PickFolderArgs {
        title: String,
        dir: PathBuf,
    }

    invoke(
        "pick_folder_with_location",
        PickFolderArgs {
            title: title.into(),
            dir: dir.into(),
        },
    )
    .await
}
