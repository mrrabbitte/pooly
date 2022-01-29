use std::error::Error;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use sled;
use sled::{Db, IVec};

use crate::models::connections::ConnectionConfig;
use crate::models::errors::ConnectionConfigError;

pub struct ConnectionConfigService {

    configs: Db

}

impl ConnectionConfigService {

    pub fn new() -> Self {
        let configs = sled::open("./stored/pooly_configs").unwrap();

        ConnectionConfigService {
            configs
        }
    }

    pub fn get(&self, db_name: &str) -> Result<Option<ConnectionConfig>, ConnectionConfigError> {
        match self.configs.get(db_name) {
            Ok(None) => Ok(None),
            Ok(Some(config_bytes)) =>
                Ok(Some(bincode::deserialize(config_bytes.as_ref())?)),
            Err(err) => Err(ConnectionConfigError::ConfigStorageError(err.to_string()))
        }
    }

    pub fn put(&self, config: ConnectionConfig) -> Result<(), ConnectionConfigError> {
        self.configs.insert(
            config.db_name.clone(),
            bincode::serialize(&config)?)?;

        self.configs.flush()?;

        Ok(())
    }

}