use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, TypedDao};
use crate::models::access::{ConnectionAccessControlEntry, LiteralConnectionIdAccessEntry, WildcardPatternConnectionIdAccessEntry};
use crate::models::access::ConnectionIdAccessEntry;
use crate::models::errors::StorageError;
use crate::models::versioned::Versioned;
use crate::models::wildcards::WildcardPattern;
use crate::services::auth::connection_ids::{LiteralConnectionIdAccessEntryService, WildcardPatternConnectionIdAccessEntryService};
use crate::services::secrets::LocalSecretsService;
use crate::services::secrets::SecretsService;
use crate::UpdatableService;

pub struct AccessControlService {

    literal_ids_service: Arc<LiteralConnectionIdAccessEntryService>,
    patterns_service: Arc<WildcardPatternConnectionIdAccessEntryService>

}

impl AccessControlService {

    pub fn new(literal_ids_service: Arc<LiteralConnectionIdAccessEntryService>,
               patterns_service: Arc<WildcardPatternConnectionIdAccessEntryService>) -> AccessControlService {
        AccessControlService {
            literal_ids_service,
            patterns_service
        }
    }

    pub fn is_allowed(&self,
                      client_id: &str,
                      connection_id: &str) -> Result<bool, StorageError> {
        Ok(
          match (self.literal_ids_service.get(client_id)?, self.patterns_service.get(client_id)?) {
              (None, None) => false,
              (None, Some(pattern)) =>
                  pattern.get_value().is_allowed(client_id, connection_id),
              (Some(literal), None) =>
                  literal.get_value().is_allowed(client_id, connection_id),
              (Some(literal),
                  Some(pattern)) =>
                  literal.get_value().is_allowed(client_id, connection_id)
                      || pattern.get_value().is_allowed(client_id, connection_id)
          }
        )
    }

}
