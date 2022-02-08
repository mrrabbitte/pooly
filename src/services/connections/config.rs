use std::sync::Arc;

use sled;
use sled::Db;

use crate::models::connections::{ConnectionConfig, Versioned};
use crate::models::errors::ConnectionConfigError;
use crate::services::BASE_STORAGE_PATH;
use crate::services::secrets::LocalSecretsService;

pub struct ConnectionConfigService {

    configs: Db,
    secrets_service: Arc<LocalSecretsService>

}

impl ConnectionConfigService {

    pub fn new(secrets_service: Arc<LocalSecretsService>) -> Self {
        let configs = sled::open(BASE_STORAGE_PATH.to_owned() + "/pooly_configs").unwrap();

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