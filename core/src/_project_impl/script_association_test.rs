use super::*;
use crate::types::ResourceId;

// **************************
// *** Script Association ***
// **************************

#[test]
fn script_association_new_works() {
    let id = ResourceId::new(); // fake id of script
    let script = ScriptAssociation::new(id);

    assert_eq!(true, script.autorun, "autorun should default to true");
    assert_eq!(0, script.priority, "order should default to 0");
}

#[test]
fn script_association_into_run_parameters_should_work() {
    // setup
    let id = ResourceId::new(); // fake id of script
    let mut script = ScriptAssociation::new(id);
    script.autorun = false;
    script.priority = 1;

    // test
    let params: RunParameters = script.clone().into();
    assert_eq!(
        &script.autorun, &params.autorun,
        "converted autorun should match"
    );
    assert_eq!(
        &script.priority, &params.priority,
        "converted priority should be correct"
    );
}

// **********************
// *** Run Parameters ***
// **********************

#[test]
fn run_parameters_new_works() {
    let params = RunParameters::new();
    assert_eq!(true, params.autorun, "autorun should default to true");
    assert_eq!(0, params.priority, "priority should default to 0");
}

#[test]
fn run_parameteres_to_association_should_work() {
    let rid = ResourceId::new();
    let params = RunParameters::new();
    let assoc = params.clone().to_association(rid.clone());

    assert_eq!(rid, assoc.script, "scripts should match");
    assert_eq!(params.autorun, assoc.autorun, "autoruns should match");
    assert_eq!(params.priority, assoc.priority, "priorities should match");
}

#[test]
fn run_parameters_ordering_should_work() {
    let p0 = RunParameters::new();
    let mut p1 = RunParameters::new();

    // equal `priority`, equal `autorun`
    assert_eq!(&p0, &p1, "parameters should be equal");

    // different `priority`, equal `autorun`
    p1.priority = 1;
    assert!(p0 < p1, "parameters with `autorun` equal should be ordered");
    assert!(p1 > p0, "parameters with `autorun` equal should be ordered");

    // different `priority`, different `autorun`
    p1.autorun = !p0.autorun;
    assert!(
        p0 < p1,
        "parameters with `autorun` not equal should be ordered"
    );
    assert!(
        p1 > p0,
        "parameters with `autorun` not equal should be ordered"
    );

    // equal `priority`, different `autorun`
    p1.priority = p0.priority;
    assert!(
        !(p0 < p1),
        "parameters with `autorun` different, and `priority` equal, should not be comparable."
    );
    assert!(
        !(p0 > p1),
        "parameters with `autorun` different, and `priority` equal, should not be comparable."
    );
    assert!(
        p0 != p1,
        "parameters with `autorun` different, and `priority` equal, should not be comparable."
    );
}
