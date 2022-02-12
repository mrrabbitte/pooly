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
               connection_id: &str) -> Result<Option<ConnectionConfig>, ConnectionConfigError> {
        match self.configs.get(connection_id) {
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

    pub fn create(&self,
                  config: ConnectionConfig) -> Result<(), ConnectionConfigError> {
        let connection_id = config.id.clone();

        let serialized = bincode::serialize(&Versioned::V1(config))?;

        self.configs.compare_and_swap(
            connection_id,
            None as Option<&[u8]>,
            Some(
                bincode::serialize(
                    &self.secrets_service.encrypt(&serialized)?
                )?
            )
        )??;

        self.configs.flush()?;

        Ok(())
    }

    pub fn clear(&self) -> Result<(), ()> {
        self.configs.clear().map_err(|_| ())
    }

}
