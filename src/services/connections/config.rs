use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use crate::models::connections::ConnectionConfig;

pub struct ConnectionConfigService {

    configs: DashMap<String, ConnectionConfig>

}

impl ConnectionConfigService {

    pub fn new() -> Self {
        let cfg = ConnectionConfig {
            hosts: vec!["localhost".to_string()],
            db_name: "pooly_test".to_string(),
            user_enc: "pooly".to_string(),
            pass_enc: "pooly_pooly_123".to_string(),
            max_connections: 10
        };

        let configs: DashMap<String, ConnectionConfig> = DashMap::new();

        configs.insert(cfg.db_name.clone(), cfg);

        ConnectionConfigService {
            configs
        }
    }

    pub fn get(&self, db_name: &str) -> Option<Ref<String, ConnectionConfig>> {
        self.configs
            .get(db_name)
    }

}