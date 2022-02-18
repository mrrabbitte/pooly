use std::sync::Arc;

use sled;
use sled::Db;

use crate::data::dao::{Dao, EncryptedDao, SimpleDao, TypedDao};
use crate::models::connections::{ConnectionConfig, VersionedConnectionConfig, ZeroizeWrapper};
use crate::models::errors::ConnectionConfigError;
use crate::services::secrets::LocalSecretsService;

const CONNECTION_CONFIGS_KEYSPACE: &str = "connection_configs_v1";

pub struct ConnectionConfigService {

    dao: TypedDao<ConnectionConfig>

}

impl ConnectionConfigService {

    pub fn new(db: Arc<Db>,
               secrets_service: Arc<LocalSecretsService>) -> Self {
        ConnectionConfigService {
            dao: TypedDao::new(
                EncryptedDao::new(
                    SimpleDao::new(CONNECTION_CONFIGS_KEYSPACE, db)
                        .unwrap(),
                    secrets_service)
            )
        }
    }

    pub fn get(&self,
               connection_id: &str) -> Result<Option<VersionedConnectionConfig>, ConnectionConfigError> {
        Ok(self.dao.get(connection_id)?)
    }

    pub fn create(&self,
                  config: ConnectionConfig) -> Result<(), ConnectionConfigError> {
        let config_id = config.id.clone();

        self.dao.create(&config_id, config)?;

        Ok(())
    }

    pub fn clear(&self) -> Result<(), ()> {
        self.dao.clear()
    }

}
