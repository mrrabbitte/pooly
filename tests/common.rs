use std::collections::HashMap;

use testcontainers::core::WaitFor;
use testcontainers::Image;

use pooly::{AppContext, test_context};
use pooly::models::auth::access::LiteralConnectionIdAccessEntry;
use pooly::models::query::connections::ConnectionConfig;
use pooly::services::updatable::UpdatableService;

pub const CLIENT_ID: &str = "client-id-1";
pub const CONNECTION_ID: &str = "connection-id-1";

pub const INTERNAL_PG_PORT: u16 = 5432;

const PG_TEST_DB: &str = "pooly_test_db";
const PG_TEST_USER: &str = "pooly_test";
const PG_TEST_PD: &str = "hnaKzVVx1qMdKvmS647ps6tjZ3Jh8p98iwpDV2l";

#[cfg(test)]
pub fn build_and_initialize_services(namespace: &str) -> AppContext {
    let context = test_context::with_namespace(namespace);

    let secrets_service = &context.secrets_service;

    secrets_service
        .clear()
        .expect("Could not clear secrets.");

    let shares = secrets_service
        .initialize()
        .expect("Could not initialize.");

    let shares_service = &context.shares_service;

    for share in shares {
        shares_service.add(share.try_into().unwrap())
            .expect("Couldn't add share, too many shares.");
    }

    secrets_service.unseal()
        .expect("Could not unseal.");

    let literal_ids_service = &context.literal_ids_service;

    literal_ids_service.clear()
        .expect("Could not clear access control service");

    literal_ids_service.create(
        LiteralConnectionIdAccessEntry::one(
            CLIENT_ID, CONNECTION_ID))
        .unwrap();

    context
}

pub fn build_config(port: u16) -> ConnectionConfig {
    ConnectionConfig {
        id: CONNECTION_ID.to_owned(),
        hosts: vec!["localhost".into()],
        ports: vec![port],
        db_name: PG_TEST_DB.to_string(),
        user: PG_TEST_USER.to_string(),
        password: PG_TEST_PD.to_string(),
        max_connections: 5,
        rate_limit: None
    }
}

fn build_env_vars() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    env_vars.insert("POSTGRES_DB".into(), PG_TEST_DB.into());
    env_vars.insert("POSTGRES_USER".into(), PG_TEST_USER.into());
    env_vars.insert("POSTGRES_PASSWORD".into(), PG_TEST_PD.into());
    env_vars.insert("POSTGRES_HOST_AUTH_METHOD".into(), "trust".into());

    env_vars
}

pub fn cleanup(app_context: AppContext,
               namespace: &str) {
    app_context.secrets_service.clear().expect("Could not clear secrets.");
    app_context.connection_config_service.clear().expect("Could not clear configs.");

    test_context::clear(namespace).expect("Could not delete storage.");
}

pub fn build_postgres_image() -> Postgres14 {
    Postgres14 {
        env_vars: build_env_vars()
    }
}

pub struct Postgres14 {
    env_vars: HashMap<String, String>
}

impl Image for Postgres14 {
    type Args = ();

    fn name(&self) -> String {
        "postgres".to_owned()
    }

    fn tag(&self) -> String {
        "14-alpine".to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}