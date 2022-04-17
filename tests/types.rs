mod common;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use derivative::Derivative;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::test_runner::{TestCaseResult, TestRunner};
    use testcontainers::{clients, Docker};
    use testcontainers::images::postgres::Postgres;
    use uuid::Uuid;

    use pooly::AppContext;
    use pooly::models::payloads::{QueryRequest, TxBulkQueryRequest, ValueWrapper};
    use pooly::models::payloads::value_wrapper::Value;
    use pooly::services::queries::QueryService;
    use pooly::services::updatable::UpdatableService;

    use crate::common;

    extern crate derivative;

    const MAX_VALUES_PER_ACTION: usize = 5;

    #[tokio::test]
    async fn test_write_read_types() {
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

        let query_service = app_context.query_service;

        let mut runner = TestRunner::default();

        runner.run(&values_test_action_strategy(query_service),
                   |action| TestCaseResult::Ok(()));
    }

    fn values_test_action_strategy(query_service: Arc<QueryService>)
                                   -> impl Strategy<Value = TestValuesAction> {
        vec(value_strategy(), MAX_VALUES_PER_ACTION)
            .prop_map(
                move |values|
                    TestValuesAction::new(query_service.clone(), values))
    }

    fn value_strategy() -> impl Strategy<Value = Value> {
        prop_oneof![
            any::<bool>().prop_map(Value::Bool),
            any::<Vec<u8>>().prop_map(Value::Bytes),
            any::<f64>().prop_map(Value::Double),
            any::<i8>().prop_map(|val| val as i32).prop_map(Value::Char),
            any::<f32>().prop_map(Value::Float),
            any::<i32>().prop_map(Value::Int4),
            any::<i64>().prop_map(Value::Int8),
            any::<String>().prop_map(Value::String),
        ]
    }

    #[derive(Derivative)]
    #[derivative(Debug)]
    struct TestValuesAction {

        #[derivative(Debug="ignore")]
        service: Arc<QueryService>,

        nullable_queries: TestValueQueries,
        non_nullable_queries: TestValueQueries,

        values: Vec<Value>

    }

    impl TestValuesAction {

        fn new(query_service: Arc<QueryService>,
               values: Vec<Value>) -> Self {
            TestValuesAction {
                nullable_queries: TestValueQueries::new(&values, true),
                non_nullable_queries: TestValueQueries::new(&values, false),
                values,
                service: query_service
            }
        }

    }

    #[derive(Debug)]
    struct TestValueQueries {

        create_table: String,
        drop_table: String,
        select_query: String,

    }

    impl TestValueQueries {

        fn new(values: &Vec<Value>,
               nullable: bool) -> Self {
            let table_name = Uuid::new_v4().to_string();

            let columns_declaration =
                Self::column_declaration(&values, nullable);

            let mut create_table = format!(
                "CREATE TABLE {table_name} ({columns_declaration});",
                table_name=table_name,
                columns_declaration=columns_declaration
            );

            let drop_table = format!("DROP TABLE {table_name}", table_name=table_name);

            let select_query = format!("SELECT * FROM {table_name}", table_name=table_name);

            TestValueQueries {
                create_table,
                drop_table,
                select_query
            }
        }

        fn column_declaration(values: &Vec<Value>,
                              nullable: bool) -> String {
            let mut declaration = String::new();

            for value in values {
                if !declaration.is_empty() {
                    declaration += ", ";
                }

                match value {
                    Value::Bool(_) =>
                        Self::col(&mut declaration, "boolean", nullable),
                    Value::Bytes(_) =>
                        Self::col(&mut declaration, "bytea", nullable),
                    Value::Double(_) =>
                        Self::col(&mut declaration, "double precision", nullable),
                    Value::Char(_) =>
                        Self::col(&mut declaration, "character(1)", nullable),
                    Value::Float(_) =>
                        Self::col(&mut declaration, "real", nullable),
                    Value::Int4(_) =>
                        Self::col(&mut declaration, "integer", nullable),
                    Value::Int8(_) =>
                        Self::col(&mut declaration, "bigint", nullable),
                    Value::String(_) =>
                        Self::col(&mut declaration, "varchar", nullable),
                    Value::Json(_) =>
                        Self::col(&mut declaration, "jsonb", nullable),
                }
            }

            declaration
        }

        fn col(declaration: &mut String,
               col_type: &str,
               nullable: bool) {
            let null_declaration = {
                if nullable {
                    "null"
                } else {
                    "not null"
                }
            };

            let col_name = format!("{}_col", col_type);

            declaration.push_str(
                format!("{} {} {}", &col_name, col_type, null_declaration).as_str());
        }

    }

}