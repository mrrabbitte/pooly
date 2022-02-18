use dashmap::DashMap;

use crate::data::dao::{Dao, EncryptedDao};

use crate::models::access::ConnectionAccessControlEntry;
use crate::models::errors::StorageError;
use crate::services::secrets::SecretsService;

const CONNECTION_CONFIGS: &str = "connection_configs_v1";

pub struct AccessControlService {

    cache: DashMap<String, ConnectionAccessControlEntry>,
    connection_ids_dao: EncryptedDao,
    connection_id_patters_dao: EncryptedDao

}

impl AccessControlService {

    // pub fn is_allowed(&self,
    //                   client_id: &str,
    //                   connection_id: &str) -> Result<bool, StorageError> {
    //     Ok(self.cache
    //         .get(client_id)
    //         .map(|ace| ace.matches(client_id, connection_id))
    //         .unwrap_or(false))
    // }
    //
    // fn retrieve_ace(&self,
    //                 client_id: &str) -> Result<ConnectionAccessControlEntry, StorageError> {
    //     self.connection_ids_dao.get(client_id)?.get_value().get_value();
    // }

}
