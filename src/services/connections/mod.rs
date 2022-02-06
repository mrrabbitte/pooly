use std::sync::Arc;

use dashmap::DashMap;
use deadpool::managed::{Object, PoolConfig};
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

use crate::models::connections::ConnectionConfig;
use crate::models::errors::ConnectionError;
use crate::services::connections::config::ConnectionConfigService;

pub mod config;

pub struct ConnectionService {

    pools: DashMap<String, Pool>,
    config_service: Arc<ConnectionConfigService>

}

pub type Connection = Object<Manager>;

impl ConnectionService {

    pub fn new(connection_config_service: Arc<ConnectionConfigService>) -> Self {
        ConnectionService {
            pools: DashMap::new(),
            config_service: connection_config_service
        }
    }

    pub async fn get(&self, db_id: &str) -> Option<Result<Connection, ConnectionError>> {
        match self.pools.get(db_id) {
            Some(pool) =>
                Some(pool.get().await.map_err(ConnectionError::PoolError)),
            None => self.create_or_empty(db_id).await
        }
    }

    async fn create_or_empty(&self,
                             db_id: &str) -> Option<Result<Connection, ConnectionError>> {
        match self.config_service.get(db_id) {
            Ok(None) => Option::None,
            Ok(Some(config)) =>
                self.add_connection_pool(&config).await,
            Err(err) => Some(Err(ConnectionError::ConnectionConfigError(err)))
        }
    }

    async fn add_connection_pool(&self,
                                 connection_config: &ConnectionConfig)
                                 -> Option<Result<Connection, ConnectionError>> {
        let mut config = Config::new();

        config.dbname = Some(connection_config.db_name.clone());
        config.hosts = Some(connection_config.hosts.clone());
        config.user = Some(connection_config.user.clone());
        config.password = Some(connection_config.password.clone());
        config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
        config.pool = Some(PoolConfig::new(connection_config.max_connections as usize));

        Some(match config.create_pool(
            Some(Runtime::Tokio1),
            NoTls
        ) {
            Ok(pool) => {
                let result =
                    pool.get().await.map_err(ConnectionError::PoolError);

                if result.is_ok() {
                    self.pools.insert(connection_config.db_name.clone(), pool);
                }

                result
            },
            Err(err) => Err(ConnectionError::CreatePoolError(err))
        })
    }

}