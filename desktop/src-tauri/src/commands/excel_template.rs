//! Excel template functions.
use calamine::{open_workbook, Xlsx};
use std::path::PathBuf;
use thot_desktop_lib::excel_template;

#[tauri::command]
pub fn load_excel(path: PathBuf) -> excel_template::Workbook {
    let workbook: Xlsx<_> = open_workbook(path).unwrap();
    excel_template::Workbook::from(workbook)
}

#[tauri::command]
pub fn load_csv(path: PathBuf) -> excel_template::Spreadsheet {
    let reader = csv::Reader::from_path(path).unwrap();
    excel_template::Spreadsheet::from(reader)
}
