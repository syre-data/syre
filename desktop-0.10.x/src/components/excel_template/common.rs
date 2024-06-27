//! Common functions for spreadsheets and workbooks.
use syre_core::project::excel_template::utils::{ALPHABET, ALPHABET_LENGTH};
use syre_core::project::excel_template::{CoordinateIndex, Index, SpreadsheetColumns, WorksheetId};

pub fn column_index_from_str(label: impl AsRef<str>) -> Option<CoordinateIndex> {
    let label = label.as_ref();
    if !label.is_ascii() {
        return None;
    }

    let label = label.to_lowercase();
    let chars = label.chars().collect::<Vec<_>>();
    match chars[..] {
        [c] => ALPHABET
            .iter()
            .position(|&l| l == c)
            .map(|idx| idx as CoordinateIndex),
        [c1, c2] => {
            let Some(idx1) = ALPHABET.iter().position(|&l| l == c1) else {
                return None;
            };

            let Some(idx2) = ALPHABET.iter().position(|&l| l == c2) else {
                return None;
            };

            let Ok(idx) = ((idx1 + 1) * (ALPHABET_LENGTH as usize) + idx2).try_into() else {
                return None;
            };

            Some(idx)
        }
        _ => None,
    }
}

/// Converts an index into a column label.
/// e.g. 0 -> A, 1 -> B, 26 -> AA
///
/// # Returns
/// + `Some` if index is valid. Letters are uppercased.
/// + `None` if index exceeds maximum.
pub fn column_index_to_str(index: u32) -> Option<String> {
    const COLUMN_LABEL_MAX_INDEX: u32 = ALPHABET_LENGTH * ALPHABET_LENGTH;

    if index < ALPHABET_LENGTH {
        let char = ALPHABET[index as usize];
        let char = char.to_uppercase().to_string();
        return Some(char);
    } else if index < COLUMN_LABEL_MAX_INDEX {
        let i1 = index / ALPHABET_LENGTH;
        let i2 = index % ALPHABET_LENGTH;
        assert!(i1 > 0);

        let c1 = ALPHABET[(i1 - 1) as usize];
        let c2 = ALPHABET[i2 as usize];
        return Some(format!("{}{}", c1.to_uppercase(), c2.to_uppercase()));
    } else {
        return None;
    }
}

pub fn spreadsheet_columns_to_string(columns: &SpreadsheetColumns) -> String {
    match columns {
        SpreadsheetColumns::Indices(indices) => indices
            .iter()
            .map(|index| column_index_to_str(index.clone()).unwrap())
            .collect::<Vec<_>>()
            .join(", "),

        SpreadsheetColumns::Header(headers) => todo!(),
    }
}

/// Parses a string as spreadsheet columns.
///
/// # Notes
/// + Valid input consists of comma-separated column labels.
///     e.g. A, BA
pub fn str_to_spreadsheet_columns(input: impl AsRef<str>) -> Option<SpreadsheetColumns> {
    let input = input.as_ref();
    if input.trim().is_empty() {
        return Some(SpreadsheetColumns::Indices(vec![]));
    }

    let input = input.split(",").collect::<Vec<_>>();
    let mut labels = Vec::with_capacity(input.len());
    for label in input {
        if let Some(label) = column_index_from_str(label.trim()) {
            labels.push(label);
        } else {
            return None;
        }
    }

    labels.sort();
    labels.dedup();
    Some(SpreadsheetColumns::Indices(labels))
}

pub fn worksheet_id_from_str(input: String) -> WorksheetId {
    match input.parse::<Index>() {
        Ok(idx) => WorksheetId::Index(idx),
        _ => WorksheetId::Name(input),
    }
}

pub fn worksheet_id_to_string(id: &WorksheetId) -> String {
    match id {
        WorksheetId::Index(idx) => idx.to_string(),
        WorksheetId::Name(name) => name.clone(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_data_selection_should_work() {
        // empty
        let input = "";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("empty returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![]));

        // single
        let input = "a";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("single returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![0]));

        // multiple
        let input = "a, b, c";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("multiple returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![0, 1, 2]));

        // invalid
        assert!(str_to_spreadsheet_columns("0").is_none());
        assert!(str_to_spreadsheet_columns("abc").is_none());
    }
}
