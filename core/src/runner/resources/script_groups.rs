//! [`ScriptAssociation`]s grouped by `priority`.
// use super::script_association::ScriptAssociation;
use crate::project::container::AnalysisMap;
use crate::project::AnalysisAssociation;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

/// A set of [`ScriptAssociation`]s.
pub type ScriptSet = HashSet<AnalysisAssociation>;

/// Map of priorities to [`ScritSet`]s.
pub type ScriptGroupMap = HashMap<i32, ScriptSet>;

/// [`ScriptGroup`]s keyed by priority.
pub struct ScriptGroups(ScriptGroupMap);

impl ScriptGroups {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Deref for ScriptGroups {
    type Target = ScriptGroupMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScriptGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<AnalysisMap> for ScriptGroups {
    fn from(scripts: AnalysisMap) -> Self {
        let mut groups = HashMap::new();
        for (rid, params) in scripts.into_iter() {
            let p = params.priority;
            if !groups.contains_key(&p) {
                // create priority group if needed
                groups.insert(p, HashSet::new());
            }

            // insert association into priority group
            let p_group = groups.get_mut(&p).expect("priority group should exist");
            p_group.insert(params.to_association(rid));
        }

        Self(groups)
    }
}

impl From<ScriptSet> for ScriptGroups {
    fn from(scripts: ScriptSet) -> Self {
        let mut groups = HashMap::new();
        for assoc in scripts {
            let p = assoc.priority;
            if !groups.contains_key(&p) {
                // create priority group if needed
                groups.insert(p, HashSet::new());
            }

            // insert association into priority group
            let p_group = groups.get_mut(&p).expect("priority group should exist");
            p_group.insert(assoc);
        }

        Self(groups)
    }
}

impl Into<Vec<(i32, ScriptSet)>> for ScriptGroups {
    fn into(self) -> Vec<(i32, ScriptSet)> {
        let mut v = self.0.into_iter().collect::<Vec<(i32, ScriptSet)>>();
        v.sort_by(|(p1, _g1), (p2, _g2)| p1.cmp(p2));

        v
    }
}

// @todo
// impl<'a> IntoIterator for &'a ScriptGroups {
//     type Item = (i32, ScriptGroup);
//     type IntoIter = std::slice::Iter<'a, (i32, ScriptGroup)>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         let v: Vec<(i32, ScriptGroup)> = self.into();
//         v.into_iter()
//     }
// }

#[cfg(test)]
#[path = "./script_groups_test.rs"]
mod script_groups_test;
