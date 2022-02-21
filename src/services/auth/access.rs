use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, TypedDao};
use crate::models::access::ConnectionAccessControlEntry;
use crate::models::errors::StorageError;
use crate::models::versioned::Versioned;
use crate::models::wildcards::WildcardPattern;
use crate::services::secrets::LocalSecretsService;
use crate::services::secrets::SecretsService;

const CONNECTION_IDS_KEYSPACE: &str = "connection_ids_v1";
const CONNECTION_ID_PATTERNS_KEYSPACE: &str = "connection_id_patterns_v1";

pub struct AccessControlService {

    aces: DashMap<String, ConnectionAccessControlEntry>,
    connection_ids: TypedDao<HashSet<String>>,
    connection_id_patters: TypedDao<HashSet<WildcardPattern>>

}

impl AccessControlService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Result<AccessControlService, StorageError> {
        let service = AccessControlService {
            aces: DashMap::new(),
            connection_ids: TypedDao::new(
                EncryptedDao::new(
                    SimpleDao::new(
                        CONNECTION_IDS_KEYSPACE,
                        db.clone())
                        .unwrap(),
                    secrets_service.clone())),
            connection_id_patters: TypedDao::new(
                EncryptedDao::new(
                    SimpleDao::new(
                        CONNECTION_ID_PATTERNS_KEYSPACE,
                        db.clone())
                        .unwrap(),
                    secrets_service.clone()))
        };

        service.initialize()?;

        Ok(service)
    }

    pub fn is_allowed(&self,
                      client_id: &str,
                      connection_id: &str) -> Result<bool, StorageError> {
        Ok(
            match self.aces.get(client_id) {
                None => self.retrieve_and_insert(client_id)?.matches(client_id, connection_id),
                Some(cached) =>
                    cached.value().matches(client_id, connection_id)
            }
        )
    }

    pub fn add_connection_ids(&self,
                              client_id: &str,
                              connection_ids: HashSet<String>) -> Result<(), StorageError> {
        self.connection_ids.create(client_id, &connection_ids)
    }

    pub fn update_connection_ids(&self,
                                 client_id: &str,
                                 connection_ids: Versioned<HashSet<String>>) -> Result<(), StorageError> {
        self.connection_ids.update(client_id, &connection_ids)
    }

    pub fn delete_connection_ids(&self, client_id: &str) -> Result<(), StorageError> {
        self.connection_ids.delete(client_id)
    }

    pub fn add_patterns(&self,
                        client_id: &str,
                        patterns: HashSet<WildcardPattern>) -> Result<(), StorageError> {
        self.connection_id_patters.create(client_id, &patterns)?;

        Ok(())
    }

    pub fn update_patterns(&self,
                           client_id: &str,
                           patterns: Versioned<HashSet<WildcardPattern>>) -> Result<(), StorageError> {
        self.connection_id_patters.update(client_id, &patterns)?;

        Ok(())
    }

    pub fn delete_patterns(&self, client_id: &str) -> Result<(), StorageError> {
        self.connection_id_patters.delete(client_id)?;

        Ok(())
    }

    pub fn has_client_id(&self, client_id: &str) -> Result<bool, StorageError> {
        Ok(self.aces.contains_key(client_id) || !self.retrieve_ace(client_id)?.is_empty())
    }

    pub fn clear(&self) -> Result<(), ()> {
        self.connection_ids.clear().and(self.connection_id_patters.clear())
    }

    fn initialize(&self) -> Result<(), StorageError> {
        let mut client_ids = HashSet::new();

        client_ids.extend(self.connection_ids.get_all_keys()?);
        client_ids.extend(self.connection_id_patters.get_all_keys()?);

        for client_id in client_ids {
            self.retrieve_and_insert(&client_id)?;
        }

        Ok(())
    }

    fn retrieve_and_insert(&self,
                           client_id: &str) -> Result<ConnectionAccessControlEntry, StorageError> {
        let ace = self.retrieve_ace(client_id)?;

        if !ace.is_empty() {
            self.aces.insert(client_id.into(), ace.clone());
        }

        Ok(ace)
    }

    fn retrieve_ace(&self,
                    client_id: &str) -> Result<ConnectionAccessControlEntry, StorageError> {
        let connection_ids =
            self.connection_ids.get(client_id)?
                .unwrap_or(Versioned::new(HashSet::new()));

        let connection_pattern_ids =
            self.connection_id_patters.get(client_id)?
                .unwrap_or(Versioned::new(HashSet::new()));

        Ok(
            ConnectionAccessControlEntry::new(
                client_id.into(),
                connection_ids,
                connection_pattern_ids)
        )
    }

}
