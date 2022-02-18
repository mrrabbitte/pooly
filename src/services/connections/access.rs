use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, TypedDao};
use crate::LocalSecretsService;
use crate::models::access::ConnectionAccessControlEntry;
use crate::models::errors::StorageError;
use crate::models::versioned::Versioned;
use crate::models::wildcards::WildcardPattern;
use crate::services::secrets::SecretsService;

const CONNECTION_IDS_KEYSPACE: &str = "connection_ids_v1";
const CONNECTION_ID_PATTERNS_KEYSPACE: &str = "connection_id_patterns_v1";

pub struct AccessControlService {

    cache: DashMap<String, ConnectionAccessControlEntry>,
    connection_ids: TypedDao<HashSet<String>>,
    connection_id_patters: TypedDao<HashSet<WildcardPattern>>

}

impl AccessControlService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> AccessControlService {
        AccessControlService {
            cache: DashMap::new(),
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
        }
    }

    pub fn is_allowed(&self,
                      client_id: &str,
                      connection_id: &str) -> Result<bool, StorageError> {
        Ok(
            match self.cache.get(client_id) {
                None =>
                    self.retrieve_ace(client_id)?.matches(client_id, connection_id),
                Some(cached) =>
                    cached.value().matches(client_id, connection_id)
            }
        )
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
