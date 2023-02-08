//! Metadata.
use std::collections::HashMap;

pub type Metadata = HashMap<String, serde_json::Value>;

#[cfg(test)]
#[path = "./metadata_test.rs"]
mod metadata_test;
