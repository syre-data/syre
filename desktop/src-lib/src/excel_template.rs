//! Spreadsheet types.
use calamine::{CellErrorType, DataType, Reader};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use std::ops::Deref;
use thot_core::project::AssetProperties;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Spreadsheet(Vec<Vec<String>>);
impl Spreadsheet {
    pub fn from_ranges(
        values_range: calamine::Range<DataType>,
        formulas_range: calamine::Range<String>,
    ) -> Self {
        let values_end = values_range.end().unwrap_or((0, 0));
        let formulas_end = formulas_range.end().unwrap_or((0, 0));
        let n_rows = values_end.0.max(formulas_end.0) as usize + 1;
        let n_cols = values_end.1.max(formulas_end.1) as usize + 1;
        let mut template = vec![vec!["".to_string(); n_cols]; n_rows];

        for row in 0..n_rows {
            for col in 0..n_cols {
                if let Some(value) = values_range.get_value((row as u32, col as u32)) {
                    if value != &DataType::Empty {
                        template[row][col] = data_type_to_string(value);
                        continue;
                    }
                }

                if let Some(formula) = formulas_range.get_value((row as u32, col as u32)) {
                    let mut out = "=".to_string();
                    out.push_str(formula);
                    template[row][col] = out;
                }
            }
        }

        Self(template)
    }
}

impl<R> From<csv::Reader<R>> for Spreadsheet
where
    R: std::io::Read,
{
    fn from(mut reader: csv::Reader<R>) -> Self {
        let mut template = Vec::new();
        let header = reader
            .headers()
            .unwrap()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();

        let mut cols = header.len();

        template.push(header);
        for result in reader.records() {
            let record = result.unwrap();
            let row = record
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>();
            if row.len() > cols {
                cols = row.len();
            }
            template.push(row);
        }

        for row in template.iter_mut() {
            row.resize_with(cols, || "".to_string());
        }

        Self(template)
    }
}

impl Deref for Spreadsheet {
    type Target = Vec<Vec<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Worksheet {
    name: String,
    data: Spreadsheet,
}

impl Worksheet {
    pub fn new(name: String, data: Spreadsheet) -> Self {
        Self { name, data }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &Spreadsheet {
        &self.data
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Workbook(Vec<Worksheet>);
impl<RS> From<calamine::Xlsx<RS>> for Workbook
where
    RS: Read + Seek,
{
    fn from(mut workbook: calamine::Xlsx<RS>) -> Self {
        let sheet_names = workbook.sheet_names();
        let workbook = sheet_names
            .clone()
            .into_iter()
            .map(|sheet_name| {
                let data = Spreadsheet::from_ranges(
                    workbook.worksheet_range(&sheet_name).unwrap(),
                    workbook.worksheet_formula(&sheet_name).unwrap(),
                );

                Worksheet {
                    name: sheet_name,
                    data,
                }
            })
            .collect::<Vec<_>>();

        Self(workbook)
    }
}

impl Deref for Workbook {
    type Target = Vec<Worksheet>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ExcelTemplate {
    pub input_data_params: InputDataParameters,
    pub template_params: ExcelTemplateParameters,
    pub asset: AssetProperties,
}

pub struct InputDataParameters {}

pub struct ExcelTemplateParameters {}

#[derive(PartialEq, Clone, Debug)]
pub enum WorksheetId {
    Name(String),
    Index(i32),
}

#[derive(PartialEq, Clone, Debug)]
pub enum DataLabelAction {
    None,
    Insert,
    Replace,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Range {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

pub fn data_type_to_string(value: &DataType) -> String {
    match value {
        DataType::Bool(val) => val.to_string(),
        DataType::DateTime(val) => val.to_string(),
        DataType::DateTimeIso(val) => val.clone(),
        DataType::Duration(val) => val.to_string(),
        DataType::DurationIso(val) => val.clone(),
        DataType::Empty => "".to_string(),
        DataType::Error(err) => match err {
            CellErrorType::Div0 => "#DIV0!",
            CellErrorType::NA => "NA",
            CellErrorType::Name => "#NAME!",
            CellErrorType::Null => "NULL",
            CellErrorType::Num => "#NUM!",
            CellErrorType::Ref => "#REF!",
            CellErrorType::Value => "#VALUE!",
            CellErrorType::GettingData => "",
        }
        .to_string(),
        DataType::Float(val) => val.to_string(),
        DataType::Int(val) => val.to_string(),
        DataType::String(val) => val.clone(),
    }
}