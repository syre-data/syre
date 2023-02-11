use super::*;
use super::{
    ResourceIdSearchFilter as RidFilter, StandardPropertiesSearchFilter as StdPropsFilter,
};

use crate::db::dev_utils::mock_props;
use crate::types::ResourceId;
use rand::Rng;

#[test]
fn std_new_should_work() {
    let f = StandardSearchFilter::new();

    assert_eq!(f.rid, None, "rid should be None");
    assert_eq!(f.name, None, "name should be None");
    assert_eq!(f.kind, None, "kind should be None");
    assert_eq!(f.tags, None, "tags should be None");
    assert_eq!(f.metadata, None, "metadata should be None");
}

#[test]
fn std_props_new_should_work() {
    let f = StandardPropertiesSearchFilter::new();

    assert_eq!(f.name, None, "name should be None");
    assert_eq!(f.kind, None, "kind should be None");
    assert_eq!(f.tags, None, "tags should be None");
    assert_eq!(f.metadata, None, "metadata should be None");
}

#[test]
fn rid_new_should_work() {
    let f = ResourceIdSearchFilter::new();

    assert_eq!(f.rid, None, "rid should be None");
}

// @todo: Known errors in test caused by randomized values can cause failure.
#[test]
fn std_props_matches_should_work() {
    // setup
    // props
    let props0 = mock_props(None, None);

    // force p1 to have different name and kind if either is None
    let p1_name = match &props0.name {
        None => Some(true),
        Some(_) => None,
    };

    let p1_kind = match &props0.kind {
        None => Some(true),
        Some(_) => None,
    };

    let mut props1 = mock_props(p1_name, p1_kind);

    // prepend to tags to prevent clash
    props1.tags = props1
        .tags
        .into_iter()
        .map(|t| format!("pre_{t}"))
        .collect();

    // filters
    let mut name_filter = StdPropsFilter::new();
    name_filter.name = Some(props0.name.clone());

    let mut kind_filter = StdPropsFilter::new();
    kind_filter.kind = Some(props0.kind.clone());

    let mut tags_filter = StdPropsFilter::new();
    let mut tf = HashSet::new();
    for t in &props0.tags {
        if rand::random() {
            tf.insert(t.clone());
        }
    }
    tags_filter.tags = Some(tf);

    let mut md_filter = StdPropsFilter::new();
    let mut md = HashMap::new();

    // create random metadata filter from data
    // ensure at least one value is included
    let f_len = props0.metadata.len();
    let mut include: Vec<bool> = Vec::with_capacity(f_len);
    for _ in 0..f_len {
        include.push(rand::random());
    }
    if include.iter().all(|&x| !x) {
        // no true values for include
        // force one to be true
        let mut rng = rand::thread_rng();
        include[rng.gen_range(0..f_len)] = true;
    }

    let mut i = 0;
    for (k, (v, _)) in &props0.metadata {
        if include[i] {
            md.insert(k.clone(), v.clone());
        }
        i += 1;
    }
    if md.len() == 1 {
        for (k, v) in &md {
            if let serde_json::Value::Bool(_) = v {
                // ensure if matching on boolean value only
                // props1 has opposite value of props0
                let (p0_val, _) = props0.metadata.get(k).expect("metadata should exist");
                let p0_val = p0_val.as_bool().expect("metadata value should be a bool");
                let (_, inherited) = props1.metadata.get(k).expect("metadata value should exist");
                props1.metadata.insert(
                    k.clone(),
                    (serde_json::Value::Bool(!p0_val), inherited.clone()),
                );
            }
        }
    }
    md_filter.metadata = Some(md);

    // test
    // name
    assert!(name_filter.matches(&props0), "name filter should match");
    assert!(
        !name_filter.matches(&props1),
        "name filter should not match"
    );

    // kind
    assert!(kind_filter.matches(&props0), "kind filter should match");
    assert!(
        !kind_filter.matches(&props1),
        "kind filter should not match"
    );

    // tags
    assert!(tags_filter.matches(&props0), "tags filter should match");
    assert!(
        !tags_filter.matches(&props1),
        "tags filter should not match"
    );

    // metadata
    assert!(md_filter.matches(&props0), "metadata filter should match");
    assert!(
        !md_filter.matches(&props1),
        "metadata filter should not match"
    );

    // @todo: Check empty filters. Specifically for `tags` and `metadata`.
}

#[test]
fn rid_matches_should_work() {
    // setup
    let rid0 = ResourceId::new();
    let mut rid1 = ResourceId::new();

    let mut rid_filter = RidFilter::new();
    rid_filter.rid = Some(rid0.clone());

    // test
    assert!(rid_filter.matches(&rid0), "resource id filter should match");
    assert!(
        !rid_filter.matches(&rid1),
        "resource id filter should not match"
    );
}
