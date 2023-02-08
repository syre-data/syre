//! Standard properties for database objects.
use crate::project::standard_properties::StandardProperties as PrjStdProps;
use serde_json::Value as SerdeValue;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{de, Deserialize};

#[cfg(feature = "serde")]
use std::fmt;

#[cfg(feature = "serde")]
use std::marker::PhantomData;

// ****************
// *** Metadata ***
// ****************

/// Represents a metadatum.
///
/// # Fields
/// + `name`: Name of the datum.
/// + `value`: Value of the datum.
/// + `inherited`: Whether the metadatum in inherited.
pub struct Metadatum {
    pub name: String,
    pub value: SerdeValue,
    pub inherited: bool,
}

pub type MetadatumValue = (serde_json::Value, bool);
pub type Metadata = HashMap<String, MetadatumValue>;

#[cfg(feature = "serde")]
fn deserialize_metadata<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct MetadataVisitor {
        marker: PhantomData<fn() -> Metadata>,
    }

    impl MetadataVisitor {
        fn new() -> Self {
            MetadataVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de> de::Visitor<'de> for MetadataVisitor {
        type Value = Metadata;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an object")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut md = match access.size_hint() {
                None => Metadata::new(),
                Some(s) => Metadata::with_capacity(s),
            };

            while let Some((k, v)) = access.next_entry()? {
                md.insert(k, (v, false));
            }

            Ok(md)
        }
    }

    deserializer.deserialize_map(MetadataVisitor::new())
}

// ***************************
// *** Standard Properties ***
// ***************************

///  Standard properties for database objects.
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StandardProperties {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub tags: HashSet<String>,

    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_metadata"))]
    pub metadata: Metadata,
}

impl From<PrjStdProps> for StandardProperties {
    fn from(prj_std: PrjStdProps) -> Self {
        let tags = prj_std
            .tags
            .clone()
            .into_iter()
            .collect::<HashSet<String>>();

        let metadata = prj_std
            .metadata
            .clone()
            .into_iter()
            .map(|(k, v)| (k, (v, false)))
            .collect::<HashMap<String, MetadatumValue>>();

        StandardProperties {
            name: prj_std.name.clone(),
            kind: prj_std.kind.clone(),
            tags,
            metadata,
        }
    }
}

impl Hash for StandardProperties {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.kind.hash(state);
    }
}

#[cfg(test)]
#[path = "./standard_properties_test.rs"]
mod standard_properties_test;
