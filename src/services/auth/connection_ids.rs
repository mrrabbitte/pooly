use std::collections::HashSet;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use sled::Db;

use crate::LocalSecretsService;
use crate::models::auth::access::{LiteralConnectionIdAccessEntry, WildcardPatternConnectionIdAccessEntry};
use crate::models::errors::StorageError;
use crate::models::versioning::updatable::{StringSetCommand, WildcardPatternSetCommand};
use crate::models::versioning::versioned::Versioned;
use crate::services::updatable::{CacheBackedService, UpdatableService};

const LITERAL_CONNECTION_IDS: &str = "literal_connection_id_aces_v1";
const PATTERN_CONNECTION_IDS: &str = "pattern_connection_id_aces_v1";

pub struct LiteralConnectionIdAccessEntryService {

    delegate: CacheBackedService<StringSetCommand, LiteralConnectionIdAccessEntry>

}

pub struct WildcardPatternConnectionIdAccessEntryService {

    delegate: CacheBackedService<WildcardPatternSetCommand, WildcardPatternConnectionIdAccessEntry>

}

impl LiteralConnectionIdAccessEntryService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>)
               -> Result<LiteralConnectionIdAccessEntryService, StorageError> {
        Ok(
            LiteralConnectionIdAccessEntryService {
                delegate: CacheBackedService::new(
                    db, LITERAL_CONNECTION_IDS, secrets_service)?
            }
        )
    }

}

impl WildcardPatternConnectionIdAccessEntryService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>)
               -> Result<WildcardPatternConnectionIdAccessEntryService, StorageError> {
        Ok(
            WildcardPatternConnectionIdAccessEntryService {
                delegate: CacheBackedService::new(
                    db, PATTERN_CONNECTION_IDS, secrets_service)?
            }
        )
    }

}

impl UpdatableService<StringSetCommand, LiteralConnectionIdAccessEntry> for LiteralConnectionIdAccessEntryService {
    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<LiteralConnectionIdAccessEntry>>>, StorageError> {
        self.delegate.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.delegate.get_all_keys()
    }

    fn create(&self, payload: LiteralConnectionIdAccessEntry) -> Result<Versioned<LiteralConnectionIdAccessEntry>, StorageError> {
        self.delegate.create(payload)
    }

    fn update(&self, id: &str, command: StringSetCommand) -> Result<Versioned<LiteralConnectionIdAccessEntry>, StorageError> {
        self.delegate.update(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.delegate.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.delegate.clear()
    }
}

impl UpdatableService<WildcardPatternSetCommand, WildcardPatternConnectionIdAccessEntry> for WildcardPatternConnectionIdAccessEntryService {
    fn get(&self, id: &str) -> Result<Option<Ref<String, Versioned<WildcardPatternConnectionIdAccessEntry>>>, StorageError> {
        self.delegate.get(id)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.delegate.get_all_keys()
    }

    fn create(&self, payload: WildcardPatternConnectionIdAccessEntry) -> Result<Versioned<WildcardPatternConnectionIdAccessEntry>, StorageError> {
        self.delegate.create(payload)
    }

    fn update(&self, id: &str, command: WildcardPatternSetCommand) -> Result<Versioned<WildcardPatternConnectionIdAccessEntry>, StorageError> {
        self.delegate.update(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.delegate.delete(id)
    }

    fn clear(&self) -> Result<(), ()> {
        self.delegate.clear()
    }
}
