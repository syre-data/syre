//! Spreadsheet types.
use calamine::{CellErrorType, DataType, Reader};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use std::ops::Deref;
use std::path::PathBuf;
use syre_core::db::StandardSearchFilter;
use syre_core::project::{AssetProperties, Script, ScriptEnv, ScriptLang};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ExcelTemplate {
    pub input_data_params: InputDataParameters,
    pub template_params: ExcelTemplateParameters,
    pub output_asset: AssetProperties,
}

impl Into<Script> for ExcelTemplate {
    fn into(self) -> Script {
        let ExcelTemplate {
            input_data_params,
            template_params,
            output_asset,
        } = self;

        // template params
        let ExcelTemplateParameters {
            path: template_path,
            replace_range,
            data_label_action,
        } = template_params;
        let worksheet = match template_params.replace_range.worksheet {
            WorksheetId::Name(name) => name,
            WorksheetId::Index(idx) => todo!(),
        };

        let Range {
            start: replace_start,
            end: replace_end,
        } = replace_range.range;

        let (header_action, template_index) = match data_label_action {
            DataLabelAction::None => ("none", None),
            DataLabelAction::Replace => ("replace", None),
            DataLabelAction::Insert { index } => ("insert", Some(index)),
        };

        // input data params
        let InputDataParameters {
            asset_filter,
            data_selection,
            skip_rows,
        } = input_data_params;
        let data_selection = match data_selection {
            DataSelection::Spreadsheet(cols) => match cols {
                SpreadsheetColumns::Indices(indices) => indices
                    .into_iter()
                    .map(|idx| idx.to_string())
                    .collect::<Vec<_>>()
                    .join(","),

                SpreadsheetColumns::Header(header) => todo!(),
            },
        };

        let StandardSearchFilter {
            rid: _,
            name: filter_name,
            kind: filter_type,
            tags: filter_tags,
            metadata: filter_metadata,
        } = asset_filter;

        // script
        let mut env = ScriptEnv::new(ScriptLang::Python, "python3");
        env.args = vec![
            "-m".into(),
            "syre_excel_template_runner".into(),
            template_path.to_str().unwrap().into(),
            worksheet,
            replace_start.to_string(),
            replace_end.to_string(),
            data_selection,
            format!("--skip-rows {skip_rows}"),
        ];

        if let Some(template_index) = template_index {
            let template_index = template_index
                .into_iter()
                .map(|idx| idx.to_string())
                .collect::<Vec<_>>()
                .join(",");

            env.args.push(format!("--template-index {template_index}"));
        }

        if let Some(filter_name) = filter_name {
            let filter_name = filter_name.unwrap_or("''".into());
            env.args.push(format!("--filter-name {filter_name}"));
        }

        if let Some(filter_type) = filter_type {
            let filter_type = filter_type.unwrap_or("''".into());
            env.args.push(format!("--filter-type {filter_type}"));
        }

        if let Some(filter_tags) = filter_tags {
            todo!();
        }
    }
}

/// Describes the shape of the input data.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct InputDataParameters {
    /// Asset filter for input data.
    pub asset_filter: StandardSearchFilter,

    /// Where data sits in each Asset.
    pub data_selection: DataSelection,

    /// Number of rows to skip until meaningful data (i.e. header or data rows).
    pub skip_rows: u32,
}

/// Describes the shape of the template and manipulations to take.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExcelTemplateParameters {
    /// Path to the template file.
    pub path: PathBuf,

    /// Range for new data to be copied into.
    /// Existing data in this range will be removed.
    pub replace_range: WorkbookRange,

    /// How new data should labeled.
    pub data_label_action: DataLabelAction,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DataLabelAction {
    /// Do not add additional labels to data.
    None,

    /// Insert the data's labels into the template, preserving the template's.
    ///
    /// # Fields
    /// + `index`: Index columns of the template. Shifted when headers are inserted.
    Insert { index: Vec<u32> },

    /// Replace the template's labels with the data's.
    Replace,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DataSelection {
    Spreadsheet(SpreadsheetColumns),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct WorkbookRange {
    pub worksheet: WorksheetId,
    pub range: Range,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum WorksheetId {
    Name(String),
    Index(i32),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum SpreadsheetColumns {
    /// Columns are identified by data headers.
    /// A header may be multiple levels.
    Header(Vec<Vec<String>>),

    /// Columns are identified by index.
    Indices(Vec<u32>),
}

/// A track is a single column or row.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TrackId {
    Name(String),
    Index(i32),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
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
