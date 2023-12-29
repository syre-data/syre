//! Common functions for spreadsheets and workbooks.

pub fn index_to_column(index: usize) -> String {
    const ALPHABET: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();

    if index < 26 {
        (ALPHABET[index] as char).to_string()
    } else {
        let c1 = ALPHABET[index / 26] as char;
        let c2 = ALPHABET[index % 26] as char;
        let mut header = String::from(c1);
        header.push(c2);
        header
    }
}
