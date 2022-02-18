use dashmap::DashMap;

use crate::data::dao::EncryptedDao;
use crate::models::access::ConnectionAccessControlEntry;

const CONNECTION_CONFIGS: &str = "connection_configs_v1";

pub struct AccessControlService {

    dao: EncryptedDao

}

impl AccessControlService {

    pub fn is_allowed(&self,
                      client_id: &str,
                      connection_id: &str) -> bool {
        false
    }

}


struct ConnectionAccessControlEntryService {

    cache: DashMap<String, ConnectionAccessControlEntry>

}

impl ConnectionAccessControlEntryService {

    fn matches(&self,
               client_id: &str,
               connection_id: &str) -> bool {
        self.cache
            .get(client_id)
            .map(|ace| ace.matches(client_id, connection_id))
            .unwrap_or(false)
    }

}







