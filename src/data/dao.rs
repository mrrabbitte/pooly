use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sled::{Db, IVec, Subscriber, Tree};
use sled::transaction::{abort, ConflictableTransactionError};

use crate::LocalSecretsService;
use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::StorageError;
use crate::models::versioned::{Versioned, VersionedVec};

pub trait Dao<T> {

    fn get(&self, id: &str) -> Result<Option<Versioned<T>>, StorageError>;

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError>;

    fn create(&self, id: &str, payload: &T) -> Result<(), StorageError>;

    fn update(&self, id: &str, new: &Versioned<T>) -> Result<(), StorageError>;

    fn delete(&self, id: &str) -> Result<(), StorageError>;

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

}

impl Dao<Vec<u8>> for SimpleDao {

    fn get(&self,
           id: &str) -> Result<Option<VersionedVec>, StorageError> {
        match self.tree.get(id) {
            Ok(None) => Ok(None),
            Ok(Some(i_vec)) => {
                Ok( Some(bincode::deserialize(&i_vec.to_vec())? ) )
            },
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
              payload: &Vec<u8>) -> Result<(), StorageError> {
        let new = bincode::serialize(&Versioned::new(payload.clone()))?;

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

                    let updated = old.update(new.clone())
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

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.tree.remove(id)?;

        Ok(())
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

}

impl Dao<ZeroizeWrapper> for EncryptedDao {

    fn get(&self,
           id: &str) -> Result<Option<Versioned<ZeroizeWrapper>>, StorageError> {
        match self.dao.get(id) {
            Ok(Some(payload)) => {
                let decrypted = self.secrets_service.decrypt(
                    &bincode::deserialize(payload.get_value())?)?;

                Ok(Some(payload.with_new_value(decrypted)))
            },
            Ok(None) => Ok(None),
            Err(err) => Err(err)
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self,
              id: &str,
              payload: &ZeroizeWrapper) -> Result<(), StorageError> {
        self.dao.create(id,
                        &bincode::serialize(
                                     &self.secrets_service.encrypt(payload.get_value())?
                                 )?
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

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.dao.delete(id)
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

}

impl<T: Serialize + for<'de> Deserialize<'de>> Dao<T> for TypedDao<T> {

    fn get(&self,
           id: &str) -> Result<Option<Versioned<T>>, StorageError> {
        match self.dao.get(id)? {
            Some(decrypted) => {
                let versioned: T =
                    bincode::deserialize(decrypted.get_value().get_value())?;

                Ok(Some(decrypted.with_new_value(versioned)))
            },
            None => Ok(None)
        }
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self,
              id: &str,
              payload: &T) -> Result<(), StorageError> {
        let serialized = bincode::serialize(payload)?;

        self.dao.create(id, &ZeroizeWrapper::new(serialized))?;

        Ok(())
    }

    fn update(&self,
              id: &str,
              new: &Versioned<T>) -> Result<(), StorageError> {
        let serialized = bincode::serialize(new.get_value())?;

        self.dao.update(id, &new.with_new_value(ZeroizeWrapper::new(serialized)))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.dao.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }
}

fn map_to_storage_err<E>(err: E) -> ConflictableTransactionError<StorageError>
    where StorageError: std::convert::From<E> {
    ConflictableTransactionError::Abort(err.into())
}

fn wrap_storage_err(err: StorageError) -> ConflictableTransactionError<StorageError> {
    ConflictableTransactionError::Abort(err)
}
