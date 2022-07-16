use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::models::versioning::updatable::{Updatable, UpdateCommand};
use crate::models::versioning::versioned::{Versioned, VersionHeader};

pub type VersionedConnectionConfig = Versioned<ConnectionConfig>;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ConnectionConfig {

    pub id: String,
    pub hosts: Vec<String>,
    pub ports: Vec<u16>,
    pub db_name: String,
    pub user: String,
    pub password: String,
    pub max_connections: i32,
    pub rate_limit: Option<RateLimitConfig>

}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ConnectionConfigUpdateCommand {

    header: VersionHeader,
    hosts: Vec<String>,
    ports: Vec<u16>,
    db_name: String,
    user: String,
    password: String,
    max_connections: i32,
    rate_limit: Option<RateLimitConfig>
}

impl UpdateCommand for ConnectionConfigUpdateCommand {
    fn get_version_header(&self) -> &VersionHeader {
        &self.header
    }
}

impl Updatable<ConnectionConfigUpdateCommand> for ConnectionConfig {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn accept(&self, update: ConnectionConfigUpdateCommand) -> Self {
        ConnectionConfig {
            id: self.id.clone(),
            hosts: update.hosts.clone(),
            ports: update.ports.clone(),
            db_name: update.db_name.clone(),
            user: update.user.clone(),
            password: update.password.clone(),
            max_connections: update.max_connections.clone(),
            rate_limit: update.rate_limit.clone(),
        }
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RateLimitConfig {

    pub max_requests_per_period: u32,
    pub period_millis: u64

}
