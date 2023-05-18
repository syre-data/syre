/// Identifier information for Thot related to storing app data.
pub struct Identifier;

impl Identifier {
    pub fn qualifier() -> String {
        String::from("so")
    }

    pub fn organization() -> String {
        String::from("thot")
    }

    pub fn application() -> String {
        String::from("core")
    }
}

#[cfg(test)]
#[path = "./identifier_test.rs"]
mod identifier_test;
