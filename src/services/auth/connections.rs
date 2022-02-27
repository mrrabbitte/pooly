use std::collections::HashSet;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use crate::data::dao::{Dao, UpdatableTypedDao};
use crate::models::access::{LiteralConnectionIdAccessEntry, WildcardPatternConnectionIdAccessEntry};
use crate::models::errors::StorageError;
use crate::models::updatable::{StringSetCommand, WildcardPatternSetCommand};
use crate::models::versioned::Versioned;
use crate::services::updatable::UpdatableService;

pub struct LiteralConnectionIdAccessEntryService {

    cache: DashMap<String, LiteralConnectionIdAccessEntry>,
    dao: UpdatableTypedDao<StringSetCommand, LiteralConnectionIdAccessEntry>

}

pub struct WildcardPatternConnectionIdAccessEntryService {

    cache: DashMap<String, WildcardPatternConnectionIdAccessEntry>,
    dao: UpdatableTypedDao<WildcardPatternSetCommand, WildcardPatternConnectionIdAccessEntry>

}

impl UpdatableService<StringSetCommand, LiteralConnectionIdAccessEntry> for LiteralConnectionIdAccessEntryService {

    fn get(&self,
           id: &str) -> Result<Option<Ref<String, Versioned<LiteralConnectionIdAccessEntry>>>, StorageError> {
        self.dao.get(id)?;

        Err(StorageError::AlreadyExistsError)
    }

    fn get_all_keys(&self) -> Result<HashSet<String>, StorageError> {
        self.dao.get_all_keys()
    }

    fn create(&self,
              payload: LiteralConnectionIdAccessEntry)
        -> Result<Versioned<LiteralConnectionIdAccessEntry>, StorageError> {
        let id = payload.client_id.clone();
        let created = Versioned::zero_version(payload);

        self.dao.create(&id, &created)?;

        Ok(created)
    }

    fn update(&self,
              id: &str,
              command: StringSetCommand)
        -> Result<Versioned<LiteralConnectionIdAccessEntry>, StorageError> {
        self.dao.accept(id, command)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        self.dao.delete(id)?;

        Ok(())
    }

}