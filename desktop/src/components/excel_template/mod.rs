//! Interface to build Excel analysis template.
pub mod common;
pub mod create_template;
pub mod excel_template;
pub mod excel_template_builder;
pub mod spreadsheet;
pub mod workbook;

pub use create_template::CreateExcelTemplate;
pub use excel_template::ExcelTemplate;
