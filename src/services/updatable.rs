use std::collections::HashSet;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use serde::{Deserialize, Serialize};

use crate::data::dao::{Dao, UpdatableTypedDao};
use crate::models::errors::StorageError;
use crate::models::updatable::{Updatable, UpdateCommand};
use crate::models::versioned::Versioned;

pub trait UpdatableService<U: UpdateCommand, T: Updatable<U>> {

    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<T>>>, StorageError>;

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError>;

    fn create(&self, payload: T) -> Result<Versioned<T>, StorageError>;

    fn update(&self,
              id: &str,
              command: U) -> Result<Versioned<T>, StorageError>;

    fn delete(&self, id: &str) -> Result<(), StorageError>;

}

pub struct CachedUpdatableService<U: UpdateCommand, T: Updatable<U>> {

    cache: DashMap<String, Versioned<T>>,
    dao: UpdatableTypedDao<U, T>

}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de> + Clone>
    CachedUpdatableService<U, T> {

    fn upsert(&self,
              id: &str,
              new: &Versioned<T>) {
        self.cache
            .entry(id.into())
            .and_modify(|old| {
                if old.should_replace(new) {
                    *old = new.clone();
                }
            })
            .or_insert(new.clone());
    }

}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de> + Clone>
    UpdatableService<U, T> for CachedUpdatableService<U, T> {

    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<T>>>, StorageError> {
        match self.cache.get(id) {
            None => {
                match self.dao.get(id)? {
                    None => Ok(None),
                    Some(value) => {
                        self.upsert(id, &value);

                        Ok(self.cache.get(id))
                    }
                }
            }
            Some(k_v) => Ok(Some(k_v))
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self, payload: T) -> Result<Versioned<T>, StorageError> {
        let created = Versioned::zero_version(payload);

        self.dao.create(created.get_value().get_id(), &created)?;

        Ok(created)
    }

    fn update(&self, id: &str, command: U) -> Result<Versioned<T>, StorageError> {
        let updated = self.dao.accept(id, command)?;

        self.upsert(id, &updated);

        Ok(updated)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.dao.delete(id)?;

        self.cache.remove(id);

        Ok(())
    }
}
