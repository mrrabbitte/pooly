use std::collections::HashSet;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use sled;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, TypedDao, UpdatableDao};
use crate::models::connections::{ConnectionConfig, ConnectionConfigUpdateCommand, VersionedConnectionConfig};
use crate::models::errors::{ConnectionConfigError, StorageError};
use crate::models::versioned::Versioned;
use crate::services::secrets::LocalSecretsService;
use crate::services::updatable::{CacheBackedService, UpdatableService};

const CONNECTION_CONFIGS_KEYSPACE: &str = "connection_configs_v1";

pub struct ConnectionConfigService {

    delegate: CacheBackedService<ConnectionConfigUpdateCommand, ConnectionConfig>

}

impl ConnectionConfigService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Result<Self, StorageError> {
        Ok(
            ConnectionConfigService {
                delegate:
                CacheBackedService::new(db, CONNECTION_CONFIGS_KEYSPACE, secrets_service)?
            }
        )
    }

}

impl UpdatableService<ConnectionConfigUpdateCommand, ConnectionConfig> for ConnectionConfigService {
    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<ConnectionConfig>>>, StorageError> {
        self.delegate.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.delegate.get_all_keys()
    }

    fn create(&self, payload: ConnectionConfig)
              -> Result<Versioned<ConnectionConfig>, StorageError> {
        self.delegate.create(payload)
    }

    fn update(&self, id: &str, command: ConnectionConfigUpdateCommand)
              -> Result<Versioned<ConnectionConfig>, StorageError> {
        self.delegate.update(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.delegate.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.delegate.clear()
    }
}
