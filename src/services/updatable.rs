use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use serde::{Deserialize, Serialize};
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, UpdatableDao};
use crate::models::errors::StorageError;
use crate::models::versioning::updatable::{Updatable, UpdateCommand};
use crate::models::versioning::versioned::Versioned;
use crate::{LocalSecretsService, TypedDao};

pub trait UpdatableService<U: UpdateCommand, T: Updatable<U>> {

    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<T>>>, StorageError>;

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError>;

    fn create(&self, payload: T) -> Result<Versioned<T>, StorageError>;

    fn update(&self,
              id: &str,
              command: U) -> Result<Versioned<T>, StorageError>;

    fn delete(&self, id: &str) -> Result<(), StorageError>;

    fn clear(&self) -> Result<(), ()>;

}

pub struct CacheBackedService<U: UpdateCommand, T: Updatable<U>> {

    cache: DashMap<String, Versioned<T>>,
    dao: UpdatableDao<U, T>

}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de> + Clone>
CacheBackedService<U, T> {

    pub fn new(db: Arc<Db>,
               keyspace: &str,
               secrets_service: Arc<LocalSecretsService>)
               -> Result<CacheBackedService<U, T>, StorageError> {
        Ok(
            CacheBackedService {
                cache: DashMap::new(),
                dao: UpdatableDao::new(
                    TypedDao::new(
                        EncryptedDao::new(
                            SimpleDao::new(keyspace, db)?, secrets_service)
                    )
                )
            }
        )
    }

    fn upsert(&self,
              id: &str,
              new: &Versioned<T>) {
        self.cache
            .entry(id.into())
            .and_modify(|old| {
                if old.should_be_replaced(new) {
                    *old = new.clone();
                }
            })
            .or_insert_with(|| new.clone());
    }

    fn remove(&self,
              id: &str,
              removed: &Versioned<T>) {
        self.cache
            .remove_if(id,
                       |_, v| v.get_header().eq(removed.get_header()));
    }

}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de> + Clone>
UpdatableService<U, T> for CacheBackedService<U, T> {

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
        match self.dao.delete(id)? {
            None => Ok(()),
            Some(removed) => {
                self.remove(id, &removed);

                Ok(())
            }
        }
    }

    fn clear(&self) -> Result<(), ()> {
        self.dao.clear()?;
        self.cache.clear();

        Ok(())
    }
}
