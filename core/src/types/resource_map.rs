//! Functionality for id-resource maps.
use super::resource_id::ResourceId;
use std::collections::HashMap;

/// Hash map keyed by the resource's id.
pub type ResourceMap<T> = HashMap<ResourceId, T>;

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
            formatter.write_str("sequence of values with ids")
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

    /// Serialize only the values.
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
