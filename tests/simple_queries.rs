
mod common;

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::ops::Deref;

    use testcontainers::{clients, Docker};
    use testcontainers::images::postgres::Postgres;
    use uuid::Uuid;

    use pooly::{AppContext, test_context};
    use pooly::models::auth::access::LiteralConnectionIdAccessEntry;
    use pooly::models::payloads::query_response::Payload;
    use pooly::models::payloads::QueryRequest;
    use pooly::models::payloads::value_wrapper::Value;
    use pooly::models::query::connections::ConnectionConfig;
    use pooly::services::updatable::UpdatableService;

    use crate::common;



    #[tokio::test]
    async fn test_simple_query() {
        pretty_env_logger::try_init().unwrap();

        let namespace = Uuid::new_v4().to_string();

        let app_context = common::build_and_initialize_services(&namespace);

        let docker = clients::Cli::default();

        let container =
            docker
                .run(Postgres::default().with_env_vars(common::build_env_vars()));

        let pg_host = container.get_host_port(common::INTERNAL_PG_PORT).unwrap();

        app_context.connection_config_service
            .create(common::build_config(pg_host))
            .expect("Could not create config.");

        let queries = build_value_queries();

        for (query, expected_value) in queries {
            let response = app_context.query_service.query(
                common::CLIENT_ID,
                &QueryRequest{
                    connection_id: common::CONNECTION_ID.to_string(),
                    query: query.clone(),
                    params: vec![]
                },
                "corr-id-1").await;

            let payload = response.0.payload.expect("Expected payload.");

            assert!(matches!(&payload, Payload::Success(_)),
                    "Query: {:?}, payload: {:?}", &query, &payload);

            check_expected_value(payload, expected_value);
        }

        app_context.secrets_service.clear().expect("Could not clear secrets.");
        app_context.connection_config_service.clear().expect("Could not clear configs.");

        test_context::clear(&namespace).expect("Could not delete storage.");
    }

    fn build_value_queries() -> HashMap<String, Value> {
        let mut ret = HashMap::new();

        ret.insert("SELECT '{\"some\": \"value\"}'::jsonb".to_string(),
                   Value::Json("{\"some\": \"value\"}".into()));
        ret.insert("SELECT '{\"other\": \"val\"}'::json".to_string(),
                   Value::Json("{\"other\": \"val\"}".into()));
        ret.insert("SELECT 'some-str-value-1'".into(),
                   Value::String("some-str-value-1".into()));
        ret.insert("SELECT 128::int8".into(), Value::Int8(128));
        ret.insert("SELECT 3::int4".into(), Value::Int4(3));
        ret.insert("SELECT 0.2::float4".into(), Value::Float(0.2));
        ret.insert("SELECT 0.34::float8".into(), Value::Double(0.34));

        ret
    }

    fn check_expected_value(payload: Payload,
                            expected: Value) {
        match payload {
            Payload::Success(success) => {
                let row = success.rows.get(0).unwrap();

                let value_wrapper_maybe = row.values.get(0).unwrap();

                let value = &value_wrapper_maybe.value;

                match value {
                    Some(actual) =>
                        assert_eq!(&expected, actual),
                    None => panic!("Expected value.")
                }
            },
            Payload::Error(_) => panic!("Expected success.")
        }
    }
}
