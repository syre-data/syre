/// Identifier information for Thot related to storing app data.
pub struct Identifier;

impl Identifier {
    pub fn qualifier() -> String {
        String::from("com")
    }

    pub fn organization() -> String {
        String::from("Thot")
    }

    pub fn application() -> String {
        String::from("Thot Core")
    }
}

#[cfg(test)]
#[path = "./identifier_test.rs"]
mod identifier_test;
