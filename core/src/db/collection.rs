//! Defines the Collection trait for database implementations to use.
use super::error::Error as DbError;
use super::resources::object::StandardObject;
use super::resources::search_filter::{
    ResourceIdSearchFilter as RidFilter, SearchFilter, StandardSearchFilter as StdFilter,
};
use crate::types::{ResourceId, ResourceMap};
use crate::{Error, Result};
use std::collections::HashSet;

// ******************
// *** Collection ***
// ******************

/// Collections.
#[derive(Debug)]
pub struct Collection<T: StandardObject> {
    objects: ResourceMap<T>,
}

impl<T: StandardObject> Collection<T> {
    pub fn new() -> Self {
        Collection {
            objects: ResourceMap::new(),
        }
    }

    /// Finds all objects with `properties` matching the search criteria.
    pub fn find(&self, search: &StdFilter) -> HashSet<T> {
        let mut matched = self.objects.clone();
        matched.retain(|_, obj| search.matches(obj));

        let mut objs = HashSet::with_capacity(matched.len());
        for obj in matched.values() {
            objs.insert(obj.clone());
        }

        objs
    }

    /// Finds a single instance of an object with `properties` matching the search filter.
    pub fn find_one(&self, search: &StdFilter) -> Option<T> {
        let matched = self.find(search);

        // return random element
        for obj in matched.iter() {
            return Some(obj.clone());
        }

        // Empty
        None
    }

    /// Inserts a single object into the database.
    ///
    /// # Errors
    /// + If an object with the same resource id already exists.
    pub fn insert_one(&mut self, obj: T) -> Result {
        let rid = obj.id().clone();
        if self.objects.contains_key(&rid) {
            return Err(Error::DbError(DbError::AlreadyExists(rid)));
        }

        self.objects.insert(rid, obj);
        Ok(())
    }

    /// Updates an object given its universal id.
    /// The object's universal id is not altered from the object passed in for the update,
    /// however other aspects of the resource id may be.
    ///
    /// # Returns
    /// The old value.
    ///
    /// # Errors
    /// + If an object with the given universal id is not found.
    pub fn update(&mut self, uid: ResourceId, mut obj: T) -> Result<T> {
        if !self.objects.contains_key(&uid) {
            return Err(Error::DbError(DbError::DoesNotExist(uid)));
        }

        *obj.id_mut() = uid.clone();
        let old_val = self.objects.insert(uid.clone(), obj);
        let Some(old_val) = old_val else {
            // @unreachable
            return Err(Error::DbError(DbError::DoesNotExist(uid)));
        };

        Ok(old_val)
    }

    /// Updates the properties a single object.
    ///
    /// # Returns
    /// Returns the orginal value.
    ///
    /// # Errors
    /// + If no objects match the search.
    /// + If more than one object matches the search.
    pub fn update_one(&mut self, search: &RidFilter, mut obj: T) -> Result<T> {
        // serach for match
        let mut matched = self.objects.clone();
        matched.retain(|_, obj| search.matches(obj.id()));

        if matched.is_empty() {
            return Err(Error::DbError(DbError::NoMatches));
        } else if matched.len() > 1 {
            return Err(Error::DbError(DbError::MultipleMatches));
        }

        // get rid of match
        let uid = matched.into_keys().next();
        let Some(uid) = uid else {
            // @unreachable
            return Err(Error::DbError(DbError::NoMatches));
        };

        // update
        *obj.id_mut() = uid.clone();
        let old_val = self.objects.insert(uid.clone(), obj);
        if old_val.is_none() {
            // should not be reachable
            return Err(Error::DbError(DbError::DoesNotExist(uid)));
        }

        Ok(old_val.unwrap())
    }

    /// Updates an object if one is found, otherwise inserts it as new.
    /// If inserting a new object, the resource id is taken as is.
    /// If updating a previous object, the resource id aremains as the original's.
    ///
    /// # Returns
    /// `None` if the object was newly inserted, or `Some` with the old value if it was updated.
    ///
    /// # Errors
    /// + If more than one object matches the search.
    pub fn update_or_insert_one(&mut self, search: &RidFilter, obj: T) -> Result<Option<T>> {
        let mut matched = self.objects.clone();
        matched.retain(|_, obj| search.matches(obj.id()));

        match matched.len() {
            0 => {
                self.insert_one(obj)?;
                Ok(None)
            }
            1 => {
                let uid = matched.into_keys().next();
                let Some(uid) = uid else {
                    // @unreachable
                    // not sure aobut which error is best to put here.
                    return Err(Error::DbError(DbError::NoMatches));
                };

                let old_val = self.update(uid, obj)?;
                Ok(Some(old_val))
            }
            _ => Err(Error::DbError(DbError::MultipleMatches)),
        }
    }

    /// Returns the number of objects in the collection.
    pub fn len(&self) -> usize {
        self.objects.len()
    }
}

#[cfg(test)]
#[path = "./collection_test.rs"]
mod collection_test;

/* ***************************************
* Addtional functionality not yet needed.
* ***************************************

   /// Inserts a vector of objects into the database.
   pub fn insert_many(&mut self, objs: Vec<T>) -> Result {
       for obj in objs{
           if let Some(found) = self.objects.get(&obj) {
               return Err(Error::DbError(DbError::AlreadyExists(obj.properties().rid.clone())));
           }
       }

       for obj in objs {
           self.objects.insert(obj);
       }

       Ok(())
   }

   /// Replaces a single object.
   ///
   /// # Returns
   /// Returns the original object.
   pub fn replace_one(&mut self, search: StdFilter, obj: T) -> Option<T> {
       for s_obj in self.objects {
           if s_obj.properties().matches(&search) {
               self.objects.remove(&s_obj);
               self.objects.insert(obj);
               return Some(s_obj);
           }
       }

       None
   }

   /// Update all objects that match the search filter.
   ///
   /// # Returns
   /// Returns a vector of the orginal values.
   pub fn update_many(&mut self, search: StdFilter, update: T) -> Vec<T> {

   }

   /// Deletes a single object.
   ///
   /// # Returns
   /// Returns the removed object.
   pub fn delete_one(&mut self, search: StdFilter) -> Option<T> {
       for obj in self.objects {
           if obj.properties.matches(search) {

           }
       }
   }

   /// Deletes all object matching the search filter.
   ///
   /// # Returns
   /// Returns a vector of the removed objects.
   pub fn delete_many(&mut self, search: StdFilter) -> Vec<T> {

   }

* End of additional functionality.
*/
