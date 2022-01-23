use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use deadpool::managed::Object;
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod, Runtime, SslMode};
use rustls::ClientConfig;
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

pub struct ConnectionService {

    connections: DashMap<String, Pool>

}

pub type Connection = Object<Manager>;

impl ConnectionService {

    pub fn new() -> Self {
        let mut mock_connections = DashMap::new();

        mock_connections.insert("pooly_test".to_owned(), get_config()
            .create_pool(
                Some(Runtime::Tokio1),
                NoTls
            ).unwrap()
        );

        ConnectionService {
            connections: mock_connections
        }
    }

    pub async fn get(&self, db_id: &str) -> Option<Result<Connection, PoolError>> {
        match self.connections.get(db_id) {
            Some(pool) => Some(pool.get().await),
            None => None
        }
    }

}

fn get_config() -> Config {

    let mut cfg = Config::new();

    cfg.host = Some("localhost".to_owned());
    cfg.dbname = Some("pooly_test".to_owned());
    cfg.user = Some("pooly".to_owned());
    cfg.password = Some("pooly_pooly_123".to_owned());
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    cfg
}