use super::*;

// *******************
// *** Association ***
// *******************

#[test]
fn association_new_works() {
    let id = ResourceId::new(); // fake id
    let association = AnalysisAssociation::new(id);

    assert_eq!(true, association.autorun, "autorun should default to true");
    assert_eq!(0, association.priority, "order should default to 0");
}

#[test]
fn association_into_run_parameters_should_work() {
    // setup
    let id = ResourceId::new(); // fake id of script
    let mut association = AnalysisAssociation::new(id);
    association.autorun = false;
    association.priority = 1;

    // test
    let params: RunParameters = association.clone().into();
    assert_eq!(
        &association.autorun, &params.autorun,
        "converted autorun should match"
    );
    assert_eq!(
        &association.priority, &params.priority,
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

    assert_eq!(rid, assoc.analysis, "associations should match");
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
