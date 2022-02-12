use std::sync::Arc;

use sled::{Db, Tree};

use crate::LocalSecretsService;
use crate::models::connections::ZeroizeWrapper;
use crate::models::errors::StorageError;

pub trait Dao<T> {

    fn get(&self, id: &str) -> Result<Option<T>, StorageError>;

    fn create(&self, id: &str, payload: Vec<u8>) -> Result<(), StorageError>;

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
           id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        match self.tree.get(format!("{}{}", self.keyspace, id)) {
            Ok(None) => Ok(None),
            Ok(Some(i_vec)) => { Ok( Some(i_vec.to_vec()) ) }
            Err(err) => Err(StorageError::RetrievalError(err.to_string()))
        }
    }

    fn create(&self,
              id: &str,
              payload: Vec<u8>) -> Result<(), StorageError> {
        self.tree.compare_and_swap(
            id,
            None as Option<&[u8]>,
            Some(payload)
        )??;

        self.tree.flush()?;

        Ok(())
    }

    fn clear(&self) -> Result<(), ()> {
        self.tree.clear().map_err(|_| ())
    }

}

pub struct EncryptedDao {

    delegate_dao: SimpleDao,
    secrets_service: Arc<LocalSecretsService>

}

impl EncryptedDao {

    pub fn new(delegate_dao: SimpleDao,
               secrets_service: Arc<LocalSecretsService>) -> Self {
        EncryptedDao {
            delegate_dao,
            secrets_service
        }
    }

}

impl Dao<ZeroizeWrapper> for EncryptedDao {

    fn get(&self,
           id: &str) -> Result<Option<ZeroizeWrapper>, StorageError> {
        match self.delegate_dao.get(id) {
            Ok(Some(payload)) =>
                Ok(Some(
                    self.secrets_service.decrypt(
                        &bincode::deserialize(&payload)?)?)),
            Ok(None) => Ok(None),
            Err(err) => Err(err)
        }
    }

    fn create(&self,
              id: &str,
              payload: Vec<u8>) -> Result<(), StorageError> {
        self.delegate_dao.create(id,
                                 bincode::serialize(
                                        &self.secrets_service.encrypt(&payload)?
                                    )?
        )
    }

    fn clear(&self) -> Result<(), ()> {
        self.delegate_dao.clear()
    }
}
