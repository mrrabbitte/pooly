use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sled::{Db, IVec, Subscriber, Tree};
use sled::transaction::{abort, ConflictableTransactionError};

use crate::LocalSecretsService;
use crate::models::errors::StorageError;
use crate::models::updatable::{Updatable, UpdateCommand};
use crate::models::versioned;
use crate::models::versioned::{Versioned, VersionedVec};
use crate::models::zeroize::ZeroizeWrapper;

pub trait Dao<T> {

    fn get(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError>;

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError>;

    fn create(&self, id: &str, payload: &Versioned<T>) -> Result<(), StorageError>;

    fn update(&self, id: &str, new: &Versioned<T>) -> Result<(), StorageError>;

    fn delete(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError>;

    fn clear(&self) -> Result<(), ()>;

}

pub struct SimpleDao {

    keyspace: String,
    tree: Tree

}

impl SimpleDao {

    pub fn new(keyspace: &str,
               db: Arc<Db>) -> Result<Self, StorageError> {
        Ok(
            SimpleDao {
                keyspace: keyspace.into(),
                tree: db.open_tree(keyspace)?
            }
        )
    }

    fn ivec_to_versioned_vec(ivec: &IVec) -> Result<VersionedVec, StorageError> {
        Ok(bincode::deserialize(&ivec.to_vec())?)
    }

}

impl Dao<Vec<u8>> for SimpleDao {

    fn get(&self,
           id: &str) -> Result<Option<VersionedVec>, StorageError> {
        match self.tree.get(id) {
            Ok(None) => Ok(None),
            Ok(Some(ivec)) => {
                Ok( Some( SimpleDao::ivec_to_versioned_vec(&ivec)? ) )
            }
            Err(err) => Err(StorageError::RetrievalError(err.to_string()))
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        let mut keys: HashSet<String> = HashSet::new();

        for key_result in self.tree.iter().keys() {
            let key: IVec = key_result?;

            keys.insert(String::from_utf8(key.to_vec())?);
        }

        Ok( keys )
    }

    fn create(&self,
              id: &str,
              payload: &VersionedVec) -> Result<(), StorageError> {
        let new = bincode::serialize(payload)?;

        self.tree.compare_and_swap(
            id,
            None as Option<&[u8]>,
            Some(new)
        )??;

        self.tree.flush()?;

        Ok(())
    }

    fn update(&self,
              id: &str,
              new: &VersionedVec) -> Result<(), StorageError> {
        self.tree.transaction(move |tx| {
            match tx.get(id)? {
                None => abort(StorageError::CouldNotFindValueToUpdate),
                Some(old_payload) => {
                    let old: VersionedVec = bincode::deserialize(&old_payload.to_vec())
                        .map_err(map_to_storage_err)?;

                    let updated = old
                        .update_with_next_version(new.clone())
                        .map_err(wrap_storage_err)?;

                    tx.insert(id,
                              bincode::serialize(&updated)
                                  .map_err(map_to_storage_err)?)?;

                    Ok(())
                }
            }
        })?;

        self.tree.flush()?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<Option<VersionedVec>, StorageError> {
        let removed_maybe = self.tree.remove(id)?;

        match removed_maybe {
            None => Ok(None),
            Some(removed) =>
                Ok(Some(SimpleDao::ivec_to_versioned_vec(&removed)?))
        }
    }

    fn clear(&self) -> Result<(), ()> {
        self.tree.clear().map_err(|_| ())
    }

}

pub struct EncryptedDao {

    dao: SimpleDao,
    secrets_service: Arc<LocalSecretsService>

}

impl EncryptedDao {

    pub fn new(dao: SimpleDao,
               secrets_service: Arc<LocalSecretsService>) -> Self {
        EncryptedDao {
            dao,
            secrets_service
        }
    }

    fn decrypt(&self,
               payload: VersionedVec) -> Result<Versioned<ZeroizeWrapper>, StorageError> {
        let decrypted = self.secrets_service.decrypt(
            &bincode::deserialize(payload.get_value())?)?;

        Ok(payload.with_new_value(decrypted))
    }

}

impl Dao<ZeroizeWrapper> for EncryptedDao {

    fn get(&self,
           id: &str) -> Result<Option<Versioned<ZeroizeWrapper>>, StorageError> {
        match self.dao.get(id) {
            Ok(Some(payload)) => Ok(Some(self.decrypt(payload)?)),
            Ok(None) => Ok(None),
            Err(err) => Err(err)
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self,
              id: &str,
              payload: &Versioned<ZeroizeWrapper>) -> Result<(), StorageError> {
        self.dao.create(id,
                        &payload.with_new_value(
                            bincode::serialize(
                                &self.secrets_service.encrypt(
                                    payload.get_value().get_value())?
                            )?
                        )
        )
    }

    fn update(&self,
              id: &str,
              new: &Versioned<ZeroizeWrapper>) -> Result<(), StorageError> {
        let encrypted = bincode::serialize(
            &self.secrets_service.encrypt(new.get_value().get_value())?
        )?;

        self.dao.update(id, &new.with_new_value(encrypted))
    }

    fn delete(&self, id: &str) -> Result<Option<Versioned<ZeroizeWrapper>>, StorageError> {
        match self.dao.delete(id)? {
            None => Ok(None),
            Some(removed) =>
                Ok(Some(self.decrypt(removed)?))
        }
    }

    fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }
}

pub struct TypedDao<T> {

    dao_type: PhantomData<T>,
    dao: EncryptedDao

}

impl<T: Serialize + for<'de> Deserialize<'de>> TypedDao<T> {

    pub fn new(dao: EncryptedDao) -> TypedDao<T> {
        TypedDao {
            dao_type: PhantomData,
            dao
        }
    }

    fn deserialize(decrypted: Versioned<ZeroizeWrapper>) -> Result<Versioned<T>, StorageError> {
        let versioned: Versioned<T> = bincode::deserialize(decrypted.get_value().get_value())?;

        Ok( versioned )
    }

}

impl<T: Serialize + for<'de> Deserialize<'de>> Dao<T> for TypedDao<T> {

    fn get(&self,
           id: &str) -> Result<Option<Versioned<T>>, StorageError> {
        match self.dao.get(id)? {
            Some(decrypted) => {
                let versioned: Versioned<T> = TypedDao::deserialize(decrypted)?;

                Ok(Some(versioned))
            },
            None => Ok(None)
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self,
              id: &str,
              payload: &Versioned<T>) -> Result<(), StorageError> {
        let serialized = bincode::serialize(payload)?;

        self.dao.create(id,
                        &payload.with_new_value(ZeroizeWrapper::new(serialized)))?;

        Ok(())
    }

    fn update(&self,
              id: &str,
              new: &Versioned<T>) -> Result<(), StorageError> {
        let serialized = bincode::serialize(new.get_value())?;

        self.dao.update(id, &new.with_new_value(ZeroizeWrapper::new(serialized)))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError> {
        match self.dao.delete(id)? {
            None => Ok(None),
            Some(removed) =>
                Ok( Some(TypedDao::deserialize(removed)?) )
        }
    }

    fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }
}

pub struct UpdatableDao<U: UpdateCommand, T: Updatable<U>> {

    dao_type: PhantomData<U>,
    dao: TypedDao<T>

}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de>> Dao<T>
for UpdatableDao<U, T> {
    fn get(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError> {
        self.dao.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self, id: &str, payload: &Versioned<T>) -> Result<(), StorageError> {
        self.dao.create(id, payload)
    }

    fn update(&self, id: &str, new: &Versioned<T>) -> Result<(), StorageError> {
        self.dao.update(id, new)
    }

    fn delete(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError> {
        self.dao.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }
}

impl<U: UpdateCommand, T: Updatable<U> + Serialize + for<'de> Deserialize<'de>> UpdatableDao<U, T> {

    pub fn new(dao: TypedDao<T>) -> UpdatableDao<U, T> {
        UpdatableDao {
            dao_type: PhantomData,
            dao
        }
    }

    pub fn accept(&self,
                  id: &str,
                  command: U) -> Result<Versioned<T>, StorageError> {
        match self.dao.get(id)? {
            None => Err(StorageError::CouldNotFindValueToUpdate),
            Some(old) => {
                let new = versioned::update(old, command)?;

                self.dao.update(id, &new)?;

                Ok(new)
            }
        }
    }

}

fn map_to_storage_err<E>(err: E) -> ConflictableTransactionError<StorageError>
    where StorageError: std::convert::From<E> {
    ConflictableTransactionError::Abort(err.into())
}

fn wrap_storage_err(err: StorageError) -> ConflictableTransactionError<StorageError> {
    ConflictableTransactionError::Abort(err)
}
