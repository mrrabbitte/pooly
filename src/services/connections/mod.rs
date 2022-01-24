use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use deadpool::managed::{Object, PoolConfig};
use deadpool_postgres::{Config, CreatePoolError, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod, Runtime, SslMode};
use rustls::ClientConfig;
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::models::connections::ConnectionConfig;
use crate::models::errors::ConnectionError;
use crate::services::connections::config::ConnectionConfigService;

mod config;

pub struct ConnectionService {

    pools: DashMap<String, Pool>,
    config_service: ConnectionConfigService

}

pub type Connection = Object<Manager>;

impl ConnectionService {

    pub fn new() -> Self {
        ConnectionService {
            pools: DashMap::new(),
            config_service: ConnectionConfigService::new()
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
            None => Option::None,
            Some(config) =>
                self.add_connection_pool(config.value()).await
        }
    }

    async fn add_connection_pool(&self,
                                 connection_config: &ConnectionConfig)
                                 -> Option<Result<Connection, ConnectionError>> {
        let mut config = Config::new();

        config.dbname = Some(connection_config.db_name.clone());
        config.hosts = Some(connection_config.hosts.clone());
        config.user = Some(connection_config.user_enc.clone());
        config.password = Some(connection_config.pass_enc.clone());
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