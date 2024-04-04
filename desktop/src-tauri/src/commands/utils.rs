//! Utilities.

pub mod trash {
    use syre_desktop_lib::error::Trash as TrashError;

    pub fn convert_os_error(code: i32, description: String) -> TrashError {
        if cfg!(target_os = "windows") {
            convert_os_error_windows(code, description)
        } else if cfg!(target_os = "macos") {
            convert_os_error_macos(code, description)
        } else {
            TrashError::Other(description)
        }
    }

    pub fn convert_os_error_windows(code: i32, description: String) -> TrashError {
        match code {
            2 | 3 => TrashError::NotFound,
            5 => TrashError::PermissionDenied,
            _ => TrashError::Other(description),
        }
    }

    pub fn convert_os_error_macos(code: i32, description: String) -> TrashError {
        tracing::debug!(?code, ?description);
        match code {
            -10010 => TrashError::NotFound,
            _ => {
                let code_pattern = regex::Regex::new(r"\((-?\d+)\)\s*$").unwrap();
                let Some(matches) = code_pattern.captures(&description) else {
                    return TrashError::Other(description);
                };

                let extracted_code = matches.get(1).unwrap();
                let extracted_code = extracted_code.as_str().parse::<i32>().unwrap();
                match extracted_code {
                    -10010 => TrashError::NotFound,
                    _ => TrashError::Other(description),
                }
            }
        }
    }
}
