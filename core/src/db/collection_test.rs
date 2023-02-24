use super::super::dev_utils::StdObj;
use super::*;
use crate::db::search_filter::StandardSearchFilter as StdFilter;
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use has_id::HasId;

#[test]
fn new_should_work() {
    Collection::<StdObj>::new();
}

// ------------
// --- find ---
// ------------

#[test]
fn find_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();

    let kind = Some(Word(EN).fake::<String>());
    let mut o1 = StdObj::new(Some(true), Some(false));
    let mut o2 = StdObj::new(Some(true), Some(false));
    o1.props.kind = kind.clone();
    o2.props.kind = kind.clone();

    c.insert_one(o1.clone())
        .expect("insert first object should work");
    c.insert_one(o2.clone())
        .expect("insert second object should work");

    let mut kind_filter = StdFilter::default();
    kind_filter.kind = Some(kind.clone());

    let mut n1_filter = StdFilter::default();
    n1_filter.name = Some(o1.props.name.clone());

    let mut none_filter = StdFilter::default();
    none_filter.kind = Some(Some(Word(EN).fake::<String>()));

    // test
    let kf = c.find(&kind_filter);
    assert_eq!(2, kf.len(), "both objects should be found");

    let f1 = c.find(&n1_filter);
    assert_eq!(1, f1.len(), "only first object should be found");

    let nf = c.find(&none_filter);
    assert_eq!(0, nf.len(), "no objects should be found");
}

#[test]
fn find_one_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();

    let kind = Some(Word(EN).fake::<String>());
    let mut o1 = StdObj::new(Some(true), Some(false));
    let mut o2 = StdObj::new(Some(true), Some(false));
    o1.props.kind = kind.clone();
    o2.props.kind = kind.clone();

    c.insert_one(o1.clone()).expect("insert object should work");
    c.insert_one(o2.clone()).expect("insert object should work");

    let mut kind_filter = StdFilter::default();
    kind_filter.kind = Some(kind.clone());

    let mut n1_filter = StdFilter::default();
    n1_filter.name = Some(o1.props.name.clone());

    let mut none_filter = StdFilter::default();
    none_filter.kind = Some(Some(String::from("not_a_kind")));

    // test
    let kf = c.find_one(&kind_filter);
    assert!(kf.is_some(), "either object should be found");

    let f1 = c.find_one(&n1_filter);
    assert!(f1.is_some(), "only first object should be found");

    let nf = c.find_one(&none_filter);
    assert!(nf.is_none(), "no objects should be found");
}

// --------------
// --- insert ---
// --------------

#[test]
fn insert_one_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o = StdObj::new(None, None);

    // test
    c.insert_one(o).expect("insert one should work");
    assert_eq!(1, c.objects.len(), "object should be inserted");
}

#[test]
#[should_panic(expected = "AlreadyExists")]
fn insert_one_if_already_exists_should_error() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o = StdObj::new(None, None);
    c.insert_one(o.clone()).expect("insert one should work");

    // test
    c.insert_one(o).unwrap();
}

// --------------
// --- update ---
// --------------

#[test]
fn update_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o0 = StdObj::new(None, None);
    let id = o0.rid.clone();
    let n0 = o0.props.name.clone();

    let mut o1 = o0.clone();
    let n1 = Some(Word(EN).fake::<String>());
    o1.props.name = n1.clone();

    c.insert_one(o0).expect("insert one should work");

    // test
    let ov = c.update(o1).unwrap();
    assert_eq!(n0, ov.props.name, "return value should be original");

    let of = c.objects.get(&id).expect("object should be found");
    assert_eq!(n1, of.props.name, "object should be updated");
}

#[test]
#[should_panic(expected = "DoesNotExist")]
fn update_with_invalid_id_should_error() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o = StdObj::new(None, None);

    // test
    c.update(o).unwrap();
}

#[test]
fn update_one_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o0 = StdObj::new(None, None);

    let mut o1 = StdObj::new(Some(true), Some(false));
    o1.rid = o0.rid.clone();

    let o2 = StdObj::new(Some(true), Some(false));

    c.insert_one(o1.clone())
        .expect("insert first object should work");

    c.insert_one(o2.clone())
        .expect("insert second object should work");

    let mut filter = StdFilter::default();
    filter.rid = Some(o0.id().clone());

    // test
    let f1 = c.update_one(o0.clone()).expect("update should work");

    assert_eq!(
        o1.props.name, f1.props.name,
        "original value should be returned"
    );

    let of = c.objects.get(&o1.rid).expect("object should be found");
    assert_eq!(o0.props.name, of.props.name, "object should be updated");

    let on = c.objects.get(&o2.rid).expect("object should be found");
    assert_eq!(o2.props.name, on.props.name, "object should not be changed");
}

#[test]
#[should_panic(expected = "NoMatches")]
fn update_one_if_no_matches_should_error() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o0 = StdObj::new(None, None);

    // test
    c.update_one(o0).unwrap();
}

#[test]
#[should_panic(expected = "MultipleMatches")]
fn update_one_with_multiple_matches_should_error() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o0 = StdObj::new(None, None);

    let kind = Some(Word(EN).fake::<String>());
    let mut o1 = StdObj::new(Some(true), Some(false));
    let mut o2 = StdObj::new(Some(true), Some(false));
    o1.props.kind = kind.clone();
    o2.props.kind = kind.clone();

    c.insert_one(o1.clone())
        .expect("insert first object should work");

    c.insert_one(o2.clone())
        .expect("insert second object should work");

    // test
    c.update_one(o0).unwrap();
}

// ------------------------
// --- update or insert ---
// ------------------------

#[test]
fn update_or_insert_one_with_new_object_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o0 = StdObj::new(None, None);
    let oid = o0.rid.clone();

    // test
    let res = c
        .update_or_insert_one(o0)
        .expect("update or insert should work");

    assert_eq!(None, res, "object should be newly inserted");
    assert!(c.objects.contains_key(&oid), "object should be inserted");
}

#[test]
fn update_or_insert_one_with_updated_object_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let lid = Some(Word(EN).fake::<String>());
    let o0 = StdObj::new(None, None);

    let mut o1 = StdObj::new(Some(true), Some(false));

    c.insert_one(o1.clone())
        .expect("insert first object should work");

    // test
    let f1 = c
        .update_or_insert_one(o0.clone())
        .expect("insert or update should work");

    let f1 = f1.expect("old value should be returned");
    assert_eq!(
        o1.props.name, f1.props.name,
        "original value should be returned"
    );

    let of = c.objects.get(&o1.rid).expect("object should be found");

    assert_eq!(o0.props.name, of.props.name, "object should be updated");
}

// -----------
// --- len ---
// -----------

#[test]
fn len_should_work() {
    // setup
    let mut c = Collection::<StdObj>::new();
    let o1 = StdObj::new(None, None);
    let o2 = StdObj::new(None, None);

    // test
    assert_eq!(0, c.len(), "no objects should be inserted");

    c.insert_one(o1).expect("insert first object should work");
    assert_eq!(1, c.len(), "one object should be inserted");

    c.insert_one(o2).expect("insert second object should work");
    assert_eq!(2, c.len(), "two objects should be inserted");
}
