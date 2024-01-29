/// Identifier information for Syre related to storing app data.
pub struct Identifier;

impl Identifier {
    pub fn qualifier() -> String {
        String::from("ai")
    }

    pub fn organization() -> String {
        String::from("syre")
    }

    pub fn application() -> String {
        String::from("syre-core")
    }
}
