use crate::types::ResourceId;
use std::cmp::{Eq, Ordering, PartialEq, PartialOrd};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// **************************
// *** Script Association ***
// **************************

/// Represents an association between a Script and a Container.
/// Contains information on the script to be run,
/// whether the Script should be run,
/// and the order of its execution relative to the current Container.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ScriptAssociation {
    pub script: ResourceId,
    pub autorun: bool,
    pub priority: i32,
}

impl ScriptAssociation {
    pub fn new(script: ResourceId) -> Self {
        ScriptAssociation {
            script,
            autorun: true,
            priority: 0,
        }
    }
}

impl Into<RunParameters> for ScriptAssociation {
    fn into(self) -> RunParameters {
        RunParameters {
            autorun: self.autorun,
            priority: self.priority,
        }
    }
}

// **********************
// *** Run Parameters ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RunParameters {
    pub autorun: bool,
    pub priority: i32,
}

impl RunParameters {
    pub fn new() -> Self {
        RunParameters {
            autorun: true,
            priority: 0,
        }
    }

    /// Converts self into a script association for the given Script.
    pub fn to_association(self, script: ResourceId) -> ScriptAssociation {
        let mut assoc = ScriptAssociation::new(script);
        assoc.autorun = self.autorun;
        assoc.priority = self.priority;

        assoc
    }
}

impl PartialOrd for RunParameters {
    /// Ordering is based on the `priority` field.
    /// If the `priority` fields are equal and `autorun` state is equal,
    /// results in the two objects being equal.
    /// If the `priority` fields are equal, but the `autorun` fields are not
    /// equal, results in the two being uncomparable.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let me = self.priority;
        let you = other.priority;
        if me < you {
            return Some(Ordering::Less);
        } else if me > you {
            return Some(Ordering::Greater);
        } else if self.autorun == other.autorun {
            // priorities equal
            return Some(Ordering::Equal);
        }

        // priorities equal, autorun not
        // can not compare
        None
    }
}

#[cfg(test)]
#[path = "./script_association_test.rs"]
mod script_association_test;
