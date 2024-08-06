use std::path::PathBuf;
use syre_desktop_lib::command::error::IoErrorKind;

#[tauri::command]
pub fn open_file(path: PathBuf) -> Result<(), IoErrorKind> {
    let path = syre_local::common::normalize_path_separators(path);
    let path = path
        .canonicalize()
        .map_err(|err| <std::io::Error as Into<IoErrorKind>>::into(err))?;
    open::that(path).map_err(|err| err.into())
}
