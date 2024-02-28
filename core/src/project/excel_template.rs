//! Excel template.
use crate::db::StandardSearchFilter;
use crate::project::AssetProperties;
use crate::types::ResourceId;
pub use coordinates::*;
use has_id::HasId;
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

#[cfg(feature = "runner")]
use crate::runner::Runnable;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(PartialEq, HasId, Clone, Debug)]
pub struct ExcelTemplate {
    #[id]
    pub rid: ResourceId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub template: TemplateParameters,
    pub input: InputParameters,
    pub output: OutputParameters,
}

/// Describes the shape of the template and manipulations to take.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct TemplateParameters {
    /// Path to the template file.
    pub path: PathBuf,

    /// Range for new data to be copied into.
    /// Existing data in this range will be removed.
    pub replace_range: WorkbookRange,

    /// How new data should labeled.
    pub data_label_action: DataLabelAction,
}

/// Describes the shape of the input data.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Debug)]
pub struct InputParameters {
    /// Asset filter for input data.
    pub asset_filter: StandardSearchFilter,

    /// Where data sits in each Asset.
    pub data_selection: DataSelection,

    /// Number of rows to skip until meaningful data (i.e. header or data rows).
    pub skip_rows: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Debug)]
pub struct OutputParameters {
    pub path: PathBuf,
    pub properties: AssetProperties,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Debug)]
pub enum DataLabelAction {
    /// No modification to input data.
    None,

    /// Insert the input sources path as an additional header above the data.
    Insert,

    /// Replace data headers with input source's path.
    Replace,
}

#[cfg(feature = "runner")]
impl Runnable for ExcelTemplate {
    fn command(&self) -> std::process::Command {
        let &ExcelTemplate {
            rid: _,
            name: _,
            description: _,
            template,
            input,
            output,
        } = &self;

        // template params
        let TemplateParameters {
            path: template_path,
            replace_range,
            data_label_action,
        } = template;

        let worksheet = match &replace_range.sheet {
            WorksheetId::Name(name) => name,
            WorksheetId::Index(idx) => todo!(),
        };

        let template_path = template_path.to_str().unwrap().to_string();

        let Range {
            start: replace_start,
            end: replace_end,
        } = replace_range.range;

        let header_action = match data_label_action {
            DataLabelAction::None => "none",
            DataLabelAction::Replace => "replace",
            DataLabelAction::Insert => "insert",
        };

        // input data params
        let InputParameters {
            asset_filter,
            data_selection,
            skip_rows,
        } = input;

        let data_selection = match data_selection {
            DataSelection::Spreadsheet(cols) => match cols {
                SpreadsheetColumns::Indices(indices) => indices
                    .into_iter()
                    .map(|idx| idx.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),

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

        // output parameters
        let OutputParameters {
            path: output_path,
            properties: output_properties,
        } = output;

        let output_path = output_path.to_string_lossy().to_string();
        let AssetProperties {
            creator: _,
            name: output_name,
            kind: output_kind,
            description: output_description,
            tags: output_tags,
            metadata: output_metadata,
            ..
        } = output_properties;

        // command
        let mut cmd = std::process::Command::new("python3");
        cmd.args(vec![
            "-m".into(),
            "syre_excel_template_runner".into(),
            template_path.clone(),
            worksheet.clone(),
            replace_start.to_string(),
            replace_end.to_string(),
            data_selection,
            format!("--output={output_path}"),
            format!("--skip-rows={skip_rows}"),
            format!("--header-action={header_action}"),
        ]);

        if let Some(filter_name) = filter_name {
            let filter_name = filter_name.clone().unwrap_or("''".into());
            cmd.arg(format!("--filter-name={filter_name}"));
        }

        if let Some(filter_type) = filter_type {
            let filter_type = filter_type.clone().unwrap_or("''".into());
            cmd.arg(format!("--filter-type={filter_type}"));
        }

        if let Some(filter_tags) = filter_tags {
            todo!();
        }

        if let Some(output_name) = output_name {
            cmd.arg(format!("--output-name={output_name}"));
        }

        if let Some(output_description) = output_description {
            cmd.arg(format!("--output-description={output_description}"));
        }

        if let Some(output_kind) = output_kind {
            cmd.arg(format!("--output-type={output_kind}"));
        }

        if output_tags.len() > 0 {
            todo!();
        }

        if output_metadata.len() > 0 {
            todo!();
        }

        cmd
    }
}

pub mod coordinates {
    //! Coordinate types.
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::ops::{Deref, DerefMut};

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    pub type Index = u32;

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Clone, Debug)]
    pub enum DataSelection {
        Spreadsheet(SpreadsheetColumns),
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Clone, Debug)]
    pub struct WorkbookRange {
        pub sheet: WorksheetId,
        pub range: Range,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Hash, Eq, PartialEq, Clone, Debug)]
    pub struct WorkbookTrack {
        pub worksheet: WorksheetId,
        pub track: Index,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Hash, Eq, PartialEq, Clone, Debug)]
    pub struct WorkbookCoordinate {
        pub worksheet: WorksheetId,
        pub coordinate: Coordinate,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Hash, Eq, PartialEq, Clone, Debug)]
    pub enum WorksheetId {
        Name(String),
        Index(Index),
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Clone, Debug)]
    pub enum SpreadsheetColumns {
        /// Columns are identified by data headers.
        /// A header may be multiple levels.
        Header(Vec<Vec<String>>),

        /// Columns are identified by index.
        Indices(Vec<Index>),
    }

    /// A track is a single column or row.
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Clone, Debug)]
    pub enum TrackId {
        Name(String),
        Index(i32),
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Clone, Debug)]
    pub struct Range {
        pub start: Index,
        pub end: Index,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Hash, Eq, PartialEq, Clone, Debug)]
    pub struct Coordinate {
        pub row: Index,
        pub column: Index,
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    #[derive(PartialEq, Clone, Debug, Default)]
    pub struct CoordinateMap<T>(HashMap<Coordinate, T>);
    impl<T> CoordinateMap<T> {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn get_coordinate(&self, row: &Index, column: &Index) -> Option<&T> {
            self.0.get(&Coordinate {
                row: *row,
                column: *column,
            })
        }

        pub fn insert_for(&mut self, row: Index, column: Index, value: T) -> Option<T> {
            self.0.insert(Coordinate { row, column }, value)
        }
    }

    impl<T> Deref for CoordinateMap<T> {
        type Target = HashMap<Coordinate, T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for CoordinateMap<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    #[derive(PartialEq, Clone, Debug, Default)]
    pub struct WorkbookCoordinateMap<T>(HashMap<WorkbookCoordinate, T>);
    impl<T> WorkbookCoordinateMap<T> {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn get_coordinate(
            &self,
            worksheet: &WorksheetId,
            row: &Index,
            column: &Index,
        ) -> Option<&T> {
            self.0.get(&WorkbookCoordinate {
                worksheet: worksheet.clone(),
                coordinate: Coordinate {
                    row: *row,
                    column: *column,
                },
            })
        }

        /// Get all values assocated with the given worksheet.
        ///
        /// # Returns
        /// + `None` if no entries for the worksheet exist.
        pub fn get_worksheet(&self, worksheet: &WorksheetId) -> Option<CoordinateMap<&T>> {
            let mut values = CoordinateMap::new();
            for (
                WorkbookCoordinate {
                    worksheet: wid,
                    coordinate,
                },
                value,
            ) in self.iter()
            {
                if wid == worksheet {
                    values.insert(coordinate.clone(), value);
                }
            }

            if values.is_empty() {
                None
            } else {
                Some(values)
            }
        }

        pub fn insert_for(
            &mut self,
            worksheet: WorksheetId,
            row: Index,
            column: Index,
            value: T,
        ) -> Option<T> {
            self.0.insert(
                WorkbookCoordinate {
                    worksheet,
                    coordinate: Coordinate { row, column },
                },
                value,
            )
        }
    }

    impl<T> Deref for WorkbookCoordinateMap<T> {
        type Target = HashMap<WorkbookCoordinate, T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for WorkbookCoordinateMap<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    #[derive(PartialEq, Clone, Debug, Default)]
    pub struct WorkbookTrackMap<T>(HashMap<WorkbookTrack, T>);
    impl<T> WorkbookTrackMap<T> {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn get_track(&self, worksheet: WorksheetId, track: Index) -> Option<&T> {
            self.0.get(&WorkbookTrack { worksheet, track })
        }

        pub fn insert_for(&mut self, worksheet: WorksheetId, track: Index, value: T) -> Option<T> {
            self.0.insert(WorkbookTrack { worksheet, track }, value)
        }

        /// Get all values assocated with the given worksheet.
        ///
        /// # Returns
        /// + `None` if no entries for the worksheet exist.
        pub fn get_worksheet(&self, worksheet: &WorksheetId) -> Option<HashMap<Index, &T>> {
            let mut values = HashMap::new();
            for (
                WorkbookTrack {
                    worksheet: wid,
                    track,
                },
                value,
            ) in self.iter()
            {
                if wid == worksheet {
                    values.insert(track.clone(), value);
                }
            }

            if values.is_empty() {
                None
            } else {
                Some(values)
            }
        }
    }

    impl<T> Deref for WorkbookTrackMap<T> {
        type Target = HashMap<WorkbookTrack, T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for WorkbookTrackMap<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
