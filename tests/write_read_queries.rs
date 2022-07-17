mod common;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use derivative::Derivative;
    use postgres_types::Kind;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::test_runner::{TestCaseResult, TestRunner};
    use runtime::Builder;
    use testcontainers::{clients, Docker};
    use testcontainers::images::postgres::Postgres;
    use tokio::runtime;
    use tokio::runtime::Runtime;
    use uuid::Uuid;

    use pooly::AppContext;
    use pooly::models::payloads::{QueryRequest, QueryResponse, QuerySuccessResponse, tx_bulk_query_response, TxBulkQueryParams, TxBulkQueryRequest, TxBulkQueryRequestBody, TxBulkQuerySuccessResponse, ValueWrapper};
    use pooly::models::payloads::query_response::Payload;
    use pooly::models::payloads::value_wrapper::Value;
    use pooly::services::queries::QueryService;
    use pooly::services::updatable::UpdatableService;

    use crate::common;

    extern crate derivative;

    const MAX_VALUES_PER_ACTION: usize = 5;

    #[test]
    fn test_write_read_types() {
        pretty_env_logger::try_init().unwrap();

        let namespace = Uuid::new_v4().to_string();

        let app_context = common::build_and_initialize_services(&namespace);

        let docker = clients::Cli::default();

        let container =
            docker
                .run(Postgres::default().with_env_vars(common::build_env_vars()));

        let pg_host = container.get_host_port_ipv4(common::INTERNAL_PG_PORT).unwrap();

        app_context.connection_config_service
            .create(common::build_config(pg_host))
            .expect("Could not create config.");

        let query_service =
            app_context.query_service.clone();

        let mut runner = TestRunner::default();

        let runtime = Arc::new(
            Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Could not build runtime.")
        );

        let result = runner.run(
            &values_test_action_strategy(query_service, runtime),
                   |action| {
                       action.test();

                       TestCaseResult::Ok(())
                   }
        );

        common::cleanup(app_context, &namespace);

        assert_eq!(true, result.is_ok(), "Got result: {:?}", result);
    }

    fn values_test_action_strategy(query_service: Arc<QueryService>,
                                   runtime: Arc<Runtime>)
                                   -> impl Strategy<Value = TestValuesAction> {
        vec(value_strategy(), MAX_VALUES_PER_ACTION)
            .prop_map(
                move |values|
                    TestValuesAction::new(
                        query_service.clone(), values, runtime.clone()))
    }

    fn value_strategy() -> impl Strategy<Value = Value> {
        prop_oneof![
            any::<bool>().prop_map(Value::Bool),
            any::<Vec<u8>>().prop_map(Value::Bytes),
            any::<f64>().prop_map(Value::Double),
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
        runtime: Arc<Runtime>,

        nullable_queries: TestValueQueries,
        non_nullable_queries: TestValueQueries,

        values: Vec<Value>,

    }

    impl TestValuesAction {

        fn new(service: Arc<QueryService>,
               values: Vec<Value>,
               runtime: Arc<Runtime>) -> Self {
            TestValuesAction {
                service,
                runtime,
                nullable_queries: TestValueQueries::new(&values, true),
                non_nullable_queries: TestValueQueries::new(&values, false),
                values
            }
        }

        fn test(&self)  {
            self.do_test(&self.non_nullable_queries,
                         self.values
                             .iter()
                             .map(|value| ValueWrapper { value: Some(value.clone()) } )
                             .collect());

            self.do_test(&self.nullable_queries,
                         self.values
                             .iter()
                             .enumerate()
                             .map(|(idx, value)| {
                                 let value_maybe = {
                                     if idx % 2 == 0 {
                                         Some(value.clone())
                                     } else {
                                         None
                                     }
                                 };
                                 ValueWrapper { value: value_maybe}
                             })
                             .collect());
        }

        fn do_test(&self,
                   queries: &TestValueQueries,
                   params: Vec<ValueWrapper>) {
            self.do_test_single_queries(queries, &params);
            self.do_test_tx_bulk_queries(queries, &params);
        }

        fn do_test_tx_bulk_queries(&self,
                                   queries: &TestValueQueries,
                                   params: &Vec<ValueWrapper>) {
            self.execute_single_query(&queries.create_table, Vec::new());

            let insert_responses = self.execute_tx_bulk_query(vec![
                (queries.insert_query.clone(), vec![
                    params.clone(),
                    params.clone(),
                    params.clone()
                ]),
                (queries.insert_query.clone(), vec![
                    params.clone(),
                    params.clone()
                ])
            ]).responses;

            assert_eq!(2, insert_responses.len());

            assert_eq!(3, insert_responses[0].row_groups.len());
            assert_eq!(2, insert_responses[1].row_groups.len());

            let mut insert_rows = insert_responses[0].row_groups[0].rows.clone();
            insert_rows.extend(insert_responses[0].row_groups[1].rows.clone());
            insert_rows.extend(insert_responses[0].row_groups[2].rows.clone());
            insert_rows.extend(insert_responses[1].row_groups[0].rows.clone());
            insert_rows.extend(insert_responses[1].row_groups[1].rows.clone());

            for row in &insert_rows {
                assert_eq!(params, &row.values);
            }

            let select_response =
                self.execute_single_query(&queries.select_query, Vec::new());

            let select_rows = select_response.rows;

            assert_eq!(insert_rows, select_rows);

            self.execute_single_query(&queries.drop_table, Vec::new());
        }

        fn do_test_single_queries(&self,
                                  queries: &TestValueQueries,
                                  params: &Vec<ValueWrapper>) {
            self.execute_single_query(&queries.create_table, Vec::new());
            self.execute_single_query(&queries.insert_query, params.clone());

            let success_response =
                self.execute_single_query(&queries.select_query, Vec::new());

            assert_eq!(&success_response.column_names, &queries.column_names);

            assert_eq!(success_response.rows.len(), 1);

            for row in success_response.rows {
                assert_eq!(params, &row.values);
            }

            self.execute_single_query(&queries.drop_table, Vec::new());
        }

        fn execute_tx_bulk_query(&self,
                                 queries: Vec<(String, Vec<Vec<ValueWrapper>>)>) -> TxBulkQuerySuccessResponse {
            let payload = self.runtime.block_on(
                self.service.bulk_tx(
                    common::CLIENT_ID,
                    &TxBulkQueryRequest {
                        connection_id: common::CONNECTION_ID.to_string(),
                        queries:
                        queries
                            .clone()
                            .into_iter()
                            .map(|(query, params)|
                                TxBulkQueryRequestBody {
                                    query: query.clone(),
                                    params: params
                                        .into_iter()
                                        .map(|values|
                                            TxBulkQueryParams { values })
                                        .collect()
                                })
                            .collect()
                    },
                    "tx_correlation_id_1"
                )
            ).0.payload;

            match payload {
                Some(tx_bulk_query_response::Payload::Success(response))
                => response,
                _ => panic!("Expected success query response, failed to execute: {:?}, got: {:?}",
                            queries, payload)
            }
        }

        fn execute_single_query(&self,
                                query: &str,
                                params: Vec<ValueWrapper>) -> QuerySuccessResponse {
            let payload = self.runtime.block_on(
                self.service.query(common::CLIENT_ID,
                                   &QueryRequest {
                                       connection_id: common::CONNECTION_ID.to_string(),
                                       query: query.into(),
                                       params
                                   },
                                   query)).0.payload;
            match payload {
                Some(Payload::Success(response)) => response,
                _ => panic!("Expected success query response, failed to execute: {}, got: {:?}",
                            query, payload)
            }
        }
    }

    #[derive(Debug)]
    struct TestValueQueries {

        column_names: Vec<String>,

        create_table: String,
        drop_table: String,

        select_query: String,
        insert_query: String

    }

    impl TestValueQueries {

        fn new(values: &Vec<Value>,
               nullable: bool) -> Self {
            let table_name = "table_".to_string()
                + Uuid::new_v4().to_string().replace("-", "_").as_str();

            let (columns_declaration, col_names) =
                Self::build_columns_declaration(&values, nullable);

            let create_table = format!(
                "CREATE TABLE {table_name} ({columns_declaration});",
                table_name=table_name,
                columns_declaration=columns_declaration
            );

            let drop_table = format!("DROP TABLE {table_name};", table_name=table_name);

            let col_names_declaration = col_names.join(",");

            let select_query = format!("SELECT {col_names_declaration} FROM {table_name};",
                                       col_names_declaration=col_names_declaration,
                                       table_name=table_name);

            let insert_query = format!(
                "INSERT INTO {table_name}({col_names_declaration}) \
                VALUES ({values_declaration}) \
                RETURNING *;",
                table_name=table_name,
                col_names_declaration=col_names_declaration,
                values_declaration=
                (1..col_names.len() + 1)
                    .map(|i| format!("${}", i))
                    .collect::<Vec<String>>()
                    .join(","));

            TestValueQueries {
                column_names: col_names,
                create_table,
                drop_table,
                select_query,
                insert_query
            }
        }

        fn build_columns_declaration(values: &Vec<Value>,
                                     nullable: bool) -> (String, Vec<String>) {
            let mut declaration = String::new();
            let mut column_names = Vec::new();

            for (idx, value) in values.iter().enumerate() {
                if !declaration.is_empty() {
                    declaration += ", ";
                }

                match value {
                    Value::Bool(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "boolean", nullable),
                    Value::Bytes(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "bytea", nullable),
                    Value::Double(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "double precision", nullable),
                    Value::Float(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "real", nullable),
                    Value::Int4(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "integer", nullable),
                    Value::Int8(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "bigint", nullable),
                    Value::String(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "varchar", nullable),
                    Value::Json(_) =>
                        Self::col(idx,
                                  &mut declaration,
                                  &mut column_names, "jsonb", nullable),
                }
            }

            (declaration, column_names)
        }

        fn col(idx: usize,
               declaration: &mut String,
               col_names: &mut Vec<String>,
               col_type: &str,
               nullable: bool) {
            let null_declaration = {
                if nullable {
                    "null"
                } else {
                    "not null"
                }
            };

            let col_name = format!("{}_{}_col", col_type.replace(" ", "_"), idx);

            declaration.push_str(
                format!("{} {} {}", &col_name, col_type, null_declaration).as_str());
            col_names.push(col_name);
        }

    }

}