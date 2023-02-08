use serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, PartialOrd};
use std::mem::{self, Discriminant};

///  Priority level of the settings.
///  Settings defined in lower priority settings are overwritten by higher ones.
///
/// # Variants
/// 0. `System`
/// 1. `User`
/// 2. `Local`
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug)]
pub enum Priority {
    System = 0,
    User = 1,
    Local = 2,
}

impl Priority {
    pub fn priority(&self) -> Discriminant<Priority> {
        mem::discriminant(&self)
    }
}

#[cfg(test)]
#[path = "./types_test.rs"]
mod types_test;
