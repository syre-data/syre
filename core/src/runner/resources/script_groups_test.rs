use super::*;
use crate::project::ScriptAssociation;
use crate::types::{ResourceId, ResourcePath};
use dev_utils::fs::TempDir;
use rand::Rng;
use std::collections::HashSet;

#[test]
fn from_hashset_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("setup shoudl work");
    let assocs = create_script_associations(&mut _dir);
    let groups = ScriptGroups::from(assocs);

    // test
    for (p, grp) in groups.0.iter() {
        for assoc in grp {
            assert_eq!(
                p, &assoc.priority,
                "association's priority should match group's"
            );
        }
    }
}

#[test]
fn into_vec_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("setup shoudl work");
    let assocs = create_script_associations(&mut _dir);
    let groups = ScriptGroups::from(assocs);

    // test
    let groups: Vec<(i32, ScriptSet)> = groups.into();
    let mut priorities = Vec::with_capacity(groups.len());
    for (p, grp) in groups {
        priorities.push(p.clone());
        for assoc in grp {
            assert_eq!(p, assoc.priority, "association priority shoudl match key");
        }
    }

    assert!(priorities.is_sorted(), "priorities should be sorted");
}

#[test]
fn iter_should_work() {
    todo!();
    //    // setup
    //    let assocs = create_script_associations();
    //    let groups = ScriptGroups::from(assocs);
    //
    //    // test
    //    let mut priorities = Vec::with_capacity(groups.0.len());
    //    for (p, _g) in groups {
    //        priorities.push(p);
    //    }
    //
    //    assert!(priorities.is_sorted(), "should visit priorities in order.");
}

// ************************
// *** helper functions ***
// ************************

fn create_script_associations(dir: &mut TempDir) -> HashSet<ScriptAssociation> {
    let mut rng = rand::thread_rng();
    let n_assocs = 20;
    let p_rng = (-n_assocs / 2)..(n_assocs / 2);

    let n_assocs = rng.gen_range(1..n_assocs);
    let mut assocs = HashSet::new();
    for _ in 0..n_assocs {
        let path = dir
            .mkfile_with_extension(".py")
            .expect("mkfile should work");

        let path = ResourcePath::new(path).expect("path to resource path should work");

        // let script = Script::new(path).expect("new script should work");
        let script = ResourceId::new();
        let priority = rng.gen_range(p_rng.clone());
        let autorun = rng.gen();

        assocs.insert(ScriptAssociation {
            script,
            priority,
            autorun,
        });
    }

    assocs
}
