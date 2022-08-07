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

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use serde::{Deserialize, Serialize};
    use serde_json;

    use crate::models::query::connections::{ConnectionConfig, ConnectionConfigUpdateCommand, RateLimitConfig};
    use crate::models::versioning::updatable::Updatable;
    use crate::models::versioning::versioned::VersionHeader;

    #[test]
    fn test_update_command() {
        let old = test_config(Some(
            RateLimitConfig {
                max_requests_per_period: 321,
                period_millis: 123
            }
        )
        );

        let command_value = vec![3,4,5];
        let command =
            ConnectionConfigUpdateCommand {
                header: VersionHeader::zero_version(),
                hosts: vec!["3".to_string(), "3".to_string(), "3".to_string()],
                ports: vec![1,2,3],
                db_name: "other-db-name-2".to_string(),
                user: "other-user-2".to_string(),
                password: "other-password-2".to_string(),
                max_connections: 999,
                rate_limit: None
            };

        let new = old.accept(command.clone());

        assert_ne!(old, new);

        assert_eq!(new.hosts, command.hosts);
        assert_eq!(new.ports, command.ports);
        assert_eq!(new.db_name, command.db_name);
        assert_eq!(new.user, command.user);
        assert_eq!(new.password, command.password);
        assert_eq!(new.max_connections, command.max_connections);
        assert_eq!(new.rate_limit, command.rate_limit);
    }

    #[test]
    fn test_serde() {
        check_serde(&test_config(None));

        check_serde(
            &test_config(Some(
                    RateLimitConfig {
                        max_requests_per_period: 321,
                        period_millis: 123
                    }
                )
            )
        );
    }

    fn check_serde<T: Serialize + PartialEq + Debug + for<'de> Deserialize<'de>>(value: &T) {
        assert_eq!(value,
                   &serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).unwrap());
    }

    fn test_config(rate_limit_config_maybe: Option<RateLimitConfig>) -> ConnectionConfig {
        ConnectionConfig{
            id: "1".to_string(),
            hosts: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            ports: vec![5532],
            db_name: "some-db-name-1".to_string(),
            user: "some-user".to_string(),
            password: "some-pass-1".to_string(),
            max_connections: 123,
            rate_limit: rate_limit_config_maybe
        }
    }

}
