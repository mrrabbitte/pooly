use std::error::Error;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use sled;
use sled::{Db, IVec};

use crate::models::connections::{ConnectionConfig, Versioned, ZeroizeWrapper};
use crate::models::errors::ConnectionConfigError;
use crate::models::secrets::EncryptedPayload;
use crate::services::secrets::SecretsService;

pub struct ConnectionConfigService {

    configs: Db,
    secrets_service: Arc<SecretsService>

}

impl ConnectionConfigService {

    pub fn new(secrets_service: Arc<SecretsService>) -> Self {
        let configs = sled::open("./stored/pooly_configs").unwrap();

        ConnectionConfigService {
            configs,
            secrets_service
        }
    }

    pub fn get(&self,
               db_name: &str) -> Result<Option<ConnectionConfig>, ConnectionConfigError> {
        match self.configs.get(db_name) {
            Ok(None) => Ok(None),
            Ok(Some(enc_bytes)) => {
                let encrypted_payload =
                    bincode::deserialize(enc_bytes.as_ref())?;

                let decrypted =
                    self.secrets_service.decrypt(&encrypted_payload)?;

                let versioned_config: Versioned<ConnectionConfig> =
                    bincode::deserialize(decrypted.get_value())?;

                Ok(Some(versioned_config.unwrap()))
            }
            Err(err) => Err(ConnectionConfigError::ConfigStorageError(err.to_string()))
        }
    }

    pub fn put(&self, config: ConnectionConfig) -> Result<(), ConnectionConfigError> {
        let db_name = config.db_name.clone();

        let serialized = bincode::serialize(&Versioned::V1(config))?;

        self.configs.insert(
            db_name,
            bincode::serialize(&self.secrets_service.encrypt(&serialized)?)?
        )?;

        self.configs.flush()?;

        Ok(())
    }

}