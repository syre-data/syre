//! Resource map for local resources with a filesystem location.
use crate::error::{Error, ResourceStoreError, Result};
use serde::{Deserialize, Serialize};
use settings_manager::LocalSettings;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thot_core::types::ResourceId;

// *************
// *** types ***
// *************

pub type ResourceWrapper<T> = Arc<Mutex<T>>;

/// Types of items able to be stored.
#[derive(Clone, Debug)]
pub enum ResourceValue<T> {
    /// No value set.
    Empty,

    /// Path to load resouce from.
    Path(PathBuf),

    /// Loaded resource.
    Resource(ResourceWrapper<T>),
}

pub type LocalResourceMap<T> = HashMap<ResourceId, ResourceValue<T>>;

// **********************
// *** Resource Store ***
// **********************

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Store for local resources with an associated filesystem location.
///
/// # Store Values
/// ```mermaid
/// flowchart TD
///     start(" ") -- "insert_id(ResourceId)" --> empty(Empty)
///     empty -- "insert_path(ResourcId, PathBuf)" --> path("Path(PathBuf)")
///     path -- "get_resource(ResourceId)" --> resource("Resource(Resource)")
///     start -- "insert_resource(ResourceId, Resource)" --> resource
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceStore<T>(
    #[serde(with = "serialize_local_resource_map_keys_only")] LocalResourceMap<T>,
)
where
    T: LocalSettings;

impl<T> ResourceStore<T>
where
    T: LocalSettings,
{
    pub fn new() -> ResourceStore<T> {
        ResourceStore(LocalResourceMap::<T>::new())
    }

    pub fn with_capacity(c: usize) -> ResourceStore<T> {
        ResourceStore(LocalResourceMap::<T>::with_capacity(c))
    }

    /// Inserts a [`ResourceId`] with an [`ResourceValue::Empty`] if the
    /// `ResourceId` is not already present, otherwise does nothing.
    pub fn insert_id(&mut self, rid: ResourceId) {
        if self.0.contains_key(&rid) {
            return;
        }

        self.insert(rid, ResourceValue::Empty);
    }

    /// Associates a path with a `ResourceId` by setting its value to a
    /// [`ResourceValue::Path`].
    ///
    /// # Returns
    /// Previous value of the path if it was a [`ResourceValue::Path`] or `None` otherwise.
    ///
    /// # Errors
    /// + [`ResourceStoreError::ResourceAlreadyLoaded`]: If the resource is already loaded,
    ///     i.e. Has a [`ResourceValue::Resource`] associated with it.
    pub fn insert_path(&mut self, rid: ResourceId, path: PathBuf) -> Result<Option<PathBuf>> {
        if let Some(ResourceValue::Resource(_)) = self.get(&rid) {
            return Err(Error::ResourceStoreError(
                ResourceStoreError::ResourceAlreadyLoaded,
            ));
        }

        let o_val = self.insert(rid.clone(), ResourceValue::Path(path));
        match o_val {
            Some(ResourceValue::Resource(_)) => panic!("Resource already set"),
            Some(ResourceValue::Path(p)) => Ok(Some(p)),
            Some(ResourceValue::Empty) => Ok(None),
            None => Ok(None),
        }
    }

    /// Gets the resourcs associated to the given [`ResourceId`].
    ///
    /// # Returns
    /// Reference to the resource with the given [`ResourceId`],
    /// or `None` if it is not a valid key.
    ///
    /// # Errors
    /// + [`ResourceStoreError::LoadEmptyValue`]: If the value is [`ResourceValue::Empty`].
    pub fn get_resource(&mut self, rid: &ResourceId) -> Option<Result<Arc<Mutex<T>>>> {
        let res = self.get(rid);
        let Some(res) = res else {
            // id not found
            return None;
        };

        match res {
            ResourceValue::Empty => Some(Err(Error::ResourceStoreError(
                ResourceStoreError::LoadEmptyValue,
            ))),

            ResourceValue::Resource(res) => return Some(Ok(res.clone())),

            ResourceValue::Path(path) => {
                // load resource and store in cache
                let res = T::load(&path);
                if let Err(err) = res {
                    // error loading object
                    return Some(Err(err.into()));
                };

                // store object
                let res = res.expect("could not unwrap resource");
                let res = Arc::new(Mutex::new(res));
                self.insert(rid.clone(), ResourceValue::Resource(res.clone()));

                Some(Ok(res))
            }
        }
    }

    /// Associates a [`ResourceId`] with a [`ResourceValue::Resource`].
    ///
    /// # Returns
    /// The previous value if it was a [`ResourceValue::Resource`] or `None` otherwise.
    pub fn insert_resource(&mut self, rid: ResourceId, res: T) -> Option<ResourceWrapper<T>> {
        let o_val = self.insert(rid, ResourceValue::Resource(Arc::new(Mutex::new(res))));
        match o_val {
            Some(ResourceValue::Resource(res)) => Some(res),
            _ => None,
        }
    }
}

impl<T> Deref for ResourceStore<T>
where
    T: LocalSettings,
{
    type Target = LocalResourceMap<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ResourceStore<T>
where
    T: LocalSettings,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> IntoIterator for ResourceStore<T>
where
    T: LocalSettings,
{
    type Item = (ResourceId, ResourceValue<T>);
    type IntoIter = std::collections::hash_map::IntoIter<ResourceId, ResourceValue<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Serialize and deserialize resource maps only using the keys.
pub mod serialize_local_resource_map_keys_only {
    use super::{LocalResourceMap, ResourceId, ResourceValue};
    use serde::de;
    use serde::ser::{SerializeSeq, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    struct RidVisitor<S> {
        marker: PhantomData<fn() -> LocalResourceMap<S>>,
    }

    impl<S> RidVisitor<S> {
        fn new() -> Self {
            RidVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, S> de::Visitor<'de> for RidVisitor<S> {
        type Value = LocalResourceMap<S>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("resource ids")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut rmap = match seq.size_hint() {
                None => LocalResourceMap::new(),
                Some(c) => LocalResourceMap::with_capacity(c),
            };

            while let Some(rid) = seq.next_element::<ResourceId>()? {
                rmap.insert(rid, ResourceValue::Empty);
            }

            Ok(rmap)
        }
    }

    /// Serialize only the keys
    pub fn serialize<S, T>(rmap: &LocalResourceMap<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(rmap.len()))?;
        for (rid, _) in rmap.iter() {
            seq.serialize_element(&rid)?;
        }

        seq.end()
    }

    /// Deserialize from only keys, initializing values to None.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<LocalResourceMap<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RidVisitor::new())
    }
}
