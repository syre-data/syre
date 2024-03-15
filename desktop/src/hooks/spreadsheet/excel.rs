//! Load a spreadsheet.
use crate::commands::common::PathBufArgs;
use crate::invoke::invoke_result;
use std::path::PathBuf;
use syre_desktop_lib::excel_template;
use yew::prelude::*;
use yew::suspense::{use_future_with, Suspension, SuspensionResult};

#[hook]
pub fn use_excel(path: PathBuf) -> SuspensionResult<excel_template::Workbook> {
    let (s, handle) = Suspension::new();
    let workbook_state = use_state(|| None);

    use_future_with(path.clone(), {
        let workbook_state = workbook_state.setter();
        |path| async move {
            let workbook = invoke_result::<excel_template::Workbook, String>(
                "load_excel",
                PathBufArgs {
                    path: (*path).clone(),
                },
            )
            .await
            .unwrap();

            workbook_state.set(Some(workbook));
            handle.resume();
        }
    })?;

    if let Some(workbook) = workbook_state.as_ref() {
        Ok(workbook.clone())
    } else {
        Err(s)
    }
}
