// use super::*;

// #[cfg(feature = "serde")]
// use serde::{Deserialize, Serialize};

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// struct ResourceMapMock {
//     #[cfg_attr(feature = "serde", serde(with = "keys_only"))]
//     pub map: ResourceMap<bool>,
// }

// #[cfg_attr(feature = "serde", derive(Deserialize))]
// struct ResourceMapKeysMock {
//     pub map: Vec<ResourceId>,
// }

// #[cfg(feature = "serde")]
// #[test]
// fn serialize_resource_map_keys_only_serialize_should_work() {
//     // setup
//     let mut map = ResourceMap::new();

//     let k_true = ResourceId::new();
//     map.insert(k_true, true);

//     let k_false = ResourceId::new();
//     map.insert(k_false, false);

//     let m = ResourceMapMock { map };
//     let json = serde_json::to_string(&m).expect("serialization should work");

//     // test
//     let key_map: ResourceMapKeysMock =
//         serde_json::from_str(&json).expect("deserialization should work");

//     for key in m.map.keys() {
//         assert!(key_map.map.contains(key), "key should be contained");
//     }
// }

// #[cfg(feature = "serde")]
// #[test]
// fn serialize_resource_map_keys_only_deserialize_should_work() {
//     // setup
//     let mut map = ResourceMap::new();

//     let k_true = ResourceId::new();
//     map.insert(k_true, true);

//     let k_false = ResourceId::new();
//     map.insert(k_false, false);

//     let m = ResourceMapMock { map };
//     let json = serde_json::to_string(&m).expect("serialization should work");

//     // test
//     let n: ResourceMapMock = serde_json::from_str(&json).expect("deserialization should work");

//     for key in m.map.keys() {
//         let val = n.map.get(key).expect("key should exist");
//         assert_eq!(&None, val, "value should be None");
//     }
// }
