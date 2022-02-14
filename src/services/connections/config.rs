use std::sync::Arc;

use sled;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao};
use crate::models::connections::ConnectionConfig;
use crate::models::errors::ConnectionConfigError;
use crate::services::secrets::LocalSecretsService;

const CONNECTION_CONFIGS: &str = "connection_configs_v1";

pub struct ConnectionConfigService {

    dao: EncryptedDao

}

impl ConnectionConfigService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Self {
        ConnectionConfigService {
            dao: EncryptedDao::new(
                SimpleDao::new(CONNECTION_CONFIGS, db)
                    .unwrap(),
                secrets_service)
        }
    }

    pub fn get(&self,
               connection_id: &str) -> Result<Option<ConnectionConfig>, ConnectionConfigError> {
        match self.dao.get(connection_id)? {
            Some(decrypted) => {
                let versioned_config: ConnectionConfig =
                    bincode::deserialize(decrypted.get_value())?;

                Ok(Some(versioned_config))
            },
            None => Ok(None)
        }
    }

    pub fn create(&self,
                  config: ConnectionConfig) -> Result<(), ConnectionConfigError> {
        let config_id = config.id.clone();

        let serialized = bincode::serialize(&config)?;

        self.dao.create(&config_id, serialized)?;

        Ok(())
    }

    pub fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }

}
