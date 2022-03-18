
#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::ops::Deref;

    use testcontainers::{clients, Docker};
    use testcontainers::images::postgres::Postgres;

    use pooly::AppContext;
    use pooly::models::auth::access::LiteralConnectionIdAccessEntry;
    use pooly::models::connections::ConnectionConfig;
    use pooly::models::payloads::query_response::Payload;
    use pooly::models::payloads::QueryRequest;
    use pooly::services::updatable::UpdatableService;

    const CLIENT_ID: &str = "client-id-1";
    const CONNECTION_ID: &str = "connection-id-1";

    const PG_TEST_DB: &str = "pooly_test_db";
    const PG_TEST_USER: &str = "pooly_test";
    const PG_TEST_PD: &str = "hnaKzVVx1qMdKvmS647ps6tjZ3Jh8p98iwpDV2l";

    const INTERNAL_PG_PORT: u16 = 5432;

    #[tokio::test]
    async fn test_simple_query() {
        let _ = pretty_env_logger::try_init().unwrap();

        let app_context = build_and_initialize_services();

        let docker = clients::Cli::default();

        let container =
            docker.run(Postgres::default().with_env_vars(build_env_vars()));

        let pg_host = container.get_host_port(INTERNAL_PG_PORT).unwrap();

        app_context.connection_config_service.create(build_config(pg_host))
            .expect("Could not create config.");

        let response = app_context.query_service.query(
            CLIENT_ID,
            &QueryRequest{
                connection_id: CONNECTION_ID.to_string(),
                query: "SELECT 1 as int8;".to_string(),
                params: vec![]
            },
            "corr-id-1").await;

        println!("{:?}", &response.0);

        assert!(matches!(response.0.payload, Some(Payload::Success(_))));

        app_context.secrets_service.clear().expect("Could not clear secrets.");
        app_context.connection_config_service.clear().expect("Could not clear configs.");
    }

    fn build_and_initialize_services() -> AppContext {
        let context = AppContext::new();

        let secrets_service = &context.secrets_service;

        secrets_service.clear()
            .expect("Could not clear secrets.");

        let shares = secrets_service.initialize()
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

    fn build_config(port: u16) -> ConnectionConfig {
        ConnectionConfig {
            id: CONNECTION_ID.to_owned(),
            hosts: vec!["localhost".into()],
            ports: vec![port],
            db_name: PG_TEST_DB.to_string(),
            user: PG_TEST_USER.to_string(),
            password: PG_TEST_PD.to_string(),
            max_connections: 5
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
}
