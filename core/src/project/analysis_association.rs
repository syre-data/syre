use crate::types::ResourceId;
use std::cmp::{Eq, Ordering, PartialEq, PartialOrd};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents an association between a Script and a Container.
/// Contains information on the script to be run,
/// whether the Script should be run,
/// and the order of its execution relative to the current Container.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AnalysisAssociation {
    pub analysis: ResourceId,
    pub autorun: bool,
    pub priority: i32,
}

impl AnalysisAssociation {
    pub fn new(analysis: ResourceId) -> Self {
        AnalysisAssociation {
            analysis,
            autorun: true,
            priority: 0,
        }
    }

    pub fn new_with_params(analysis: ResourceId, params: RunParameters) -> Self {
        AnalysisAssociation {
            analysis,
            autorun: params.autorun,
            priority: params.priority,
        }
    }
}

impl Into<RunParameters> for AnalysisAssociation {
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
    pub fn to_association(self, script: ResourceId) -> AnalysisAssociation {
        let mut assoc = AnalysisAssociation::new(script);
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
#[path = "./analysis_association_test.rs"]
mod analysis_association_test;
