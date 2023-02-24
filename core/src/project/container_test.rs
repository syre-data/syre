use super::*;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;

#[test]
fn duplicate_with_no_children_should_work() {
    // setup
    let mut o_root = Container::default();
    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    o_root.properties.name = Some(name);

    // test
    let root = o_root.duplicate().expect("could not duplicate tree");

    assert_ne!(o_root.rid, root.rid, "`ResourceId` should not match");
    assert_eq!(
        o_root.properties, root.properties,
        "properties do not match"
    );
}

#[test]
fn duplicate_should_work() {
    // setup
    let mut o_root = Container::default();
    let mut oc1 = Container::default();
    let mut oc2 = Container::default();

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    o_root.properties.name = Some(name);

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    oc1.properties.name = Some(name);

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    oc2.properties.name = Some(name);

    o_root
        .children
        .insert(oc1.rid.clone(), Some(Arc::new(Mutex::new(oc1))));

    o_root
        .children
        .insert(oc2.rid.clone(), Some(Arc::new(Mutex::new(oc2))));

    // test
    let root = o_root.duplicate().expect("could not duplicate tree");
    assert_ne!(o_root.rid, root.rid, "`ResourceId` should not match");
    assert_eq!(
        o_root.properties, root.properties,
        "properties do not match"
    );

    assert_eq!(2, root.children.len(), "incorrect children loaded");
    // @todo: Test children better.
}

// @note: Left for future use, if needed.
// **********************
// *** Mock Container ***
// **********************
/*
struct MockContainer {
    _properties: StandardProperties,
    _children: ContainerMap,
    _assets: AssetMap,
    _scripts: ScriptMap,
}

impl MockContainer {
    pub fn new() -> Self {
        MockContainer {
            _properties: StandardProperties::new(),
            _children: ContainerMap::new(),
            _assets: AssetMap::new(),
            _scripts: ScriptMap::new(),
        }
    }
}

impl HasStandardProperties for MockContainer {
    fn properties(&self) -> &StandardProperties {
        &self._properties
    }

    fn properties_mut(&mut self) -> &mut StandardProperties {
        &mut self._properties
    }
}

impl Container for MockContainer {
    fn children(&self) -> &ContainerMap {
        &self._children
    }

    fn children_mut(&mut self) -> &mut ContainerMap {
        &mut self._children
    }

    fn assets(&self) -> &AssetMap {
        &self._assets
    }

    fn assets_mut(&mut self) -> &mut AssetMap {
        &mut self._assets
    }

    fn scripts(&self) -> &ScriptMap {
        &self._scripts
    }

    fn scripts_mut(&mut self) -> &mut ScriptMap {
        &mut self._scripts
    }
}
*/
