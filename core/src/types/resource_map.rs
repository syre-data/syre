//! Functionality for id-resource maps.
use super::resource_id::ResourceId;
use std::collections::HashMap;

/// Hash map keyed by the resource's id.
pub type ResourceMap<T> = HashMap<ResourceId, T>;

/// Hash map keyed by the resource's id.
/// If the value is `None`, the resource has not yet been loaded.
pub type ResourceStore<T> = HashMap<ResourceId, Option<T>>;

/// Serialize and deserialize maps only using the keys.
#[cfg(feature = "serde")]
pub mod keys_only {
    use serde::ser::SerializeSeq;
    use serde::{de, ser};
    use std::collections::HashMap;
    use std::fmt;
    use std::hash::Hash;
    use std::marker::PhantomData;

    struct KeyVisitor<K, V>
    where
        V: Default,
    {
        marker: PhantomData<fn() -> HashMap<K, V>>,
    }

    impl<K, V> KeyVisitor<K, V>
    where
        V: Default,
    {
        fn new() -> Self {
            KeyVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, K, V> de::Visitor<'de> for KeyVisitor<K, V>
    where
        K: Hash + Eq + de::Deserialize<'de>,
        V: Default,
    {
        type Value = HashMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map keys")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut map = match seq.size_hint() {
                None => HashMap::new(),
                Some(c) => HashMap::with_capacity(c),
            };

            while let Some(key) = seq.next_element::<K>()? {
                map.insert(key, Default::default());
            }

            Ok(map)
        }
    }

    /// Serialize only the keys
    pub fn serialize<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: ser::Serialize,
        S: ser::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (id, _) in map.iter() {
            seq.serialize_element(&id)?;
        }

        seq.end()
    }

    /// Deserialize from only keys, initializing values to default.
    pub fn deserialize<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        K: Hash + Eq + de::Deserialize<'de>,
        V: Default,
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(KeyVisitor::new())
    }
}

/// Serialize and deserialize maps only using the values.
#[cfg(feature = "serde")]
pub mod values_only {
    use has_id::HasIdSerde;
    use serde::ser::SerializeSeq;
    use serde::{de, ser};
    use std::collections::HashMap;
    use std::fmt;
    use std::marker::PhantomData;

    struct ValueVisitor<'de, V>
    where
        V: HasIdSerde<'de>,
    {
        marker: PhantomData<fn() -> HashMap<<V as HasIdSerde<'de>>::Id, V>>,
    }

    impl<'de, V> ValueVisitor<'de, V>
    where
        V: HasIdSerde<'de>,
    {
        fn new() -> Self {
            ValueVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, V> de::Visitor<'de> for ValueVisitor<'de, V>
    where
        V: de::Deserialize<'de> + HasIdSerde<'de>,
    {
        type Value = HashMap<<V as HasIdSerde<'de>>::Id, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map values")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut map = match seq.size_hint() {
                None => HashMap::new(),
                Some(c) => HashMap::with_capacity(c),
            };

            while let Some(value) = seq.next_element::<V>()? {
                map.insert(HasIdSerde::id(&value).clone(), value);
            }

            Ok(map)
        }
    }

    /// Serialize only the keys
    pub fn serialize<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        V: ser::Serialize,
        S: ser::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (_, value) in map.iter() {
            seq.serialize_element(&value)?;
        }

        seq.end()
    }

    /// Deserialize from only values getting id from it.
    pub fn deserialize<'de, D, V>(
        deserializer: D,
    ) -> Result<HashMap<<V as HasIdSerde<'de>>::Id, V>, D::Error>
    where
        V: de::Deserialize<'de> + HasIdSerde<'de>,
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ValueVisitor::new())
    }
}

#[cfg(test)]
#[path = "./resource_map_test.rs"]
mod resource_map_test;
