use std::sync::Arc;

use deadpool_postgres::Transaction;
use futures_util::future;
use postgres_types::ToSql;

use crate::models::errors::QueryError;
use crate::models::payloads::{ErrorResponse, query_response, QueryRequest, QueryResponse, RowResponseGroup, tx_bulk_query_response, TxBulkQueryRequest, TxBulkQueryRequestBody, TxBulkQueryResponse, TxBulkQuerySuccessResponse, TxQuerySuccessResponse};
use crate::models::payloads::QuerySuccessResponse;
use crate::models::query::parameters::convert_params;
use crate::models::query::rows::convert_rows;
use crate::models::responses::ResponseWithCode;
use crate::services::auth::access::AccessControlService;
use crate::services::connections::ConnectionService;

pub struct QueryService {

    access_control_service: Arc<AccessControlService>,
    connection_service: ConnectionService

}

impl QueryService {

    pub fn new(access_control_service: Arc<AccessControlService>,
               connection_service: ConnectionService) -> Self {
        QueryService {
            access_control_service,
            connection_service
        }
    }

    pub async fn bulk_tx(&self,
                         client_id: &str,
                         request: &TxBulkQueryRequest,
                         correlation_id: &str) -> ResponseWithCode<TxBulkQueryResponse> {
        match self.do_bulk_tx(client_id, request).await {
            Ok(ok) => ResponseWithCode::ok(ok.into()),
            Err(err) => QueryService::build_response(err, correlation_id)
        }
    }

    pub async fn query(&self,
                       client_id: &str,
                       request: &QueryRequest,
                       correlation_id: &str) -> ResponseWithCode<QueryResponse> {
        match self.do_query(client_id, request).await {
            Ok(ok) => ResponseWithCode::ok(ok.into()),
            Err(err) => QueryService::build_response(err, correlation_id)
        }
    }

    async fn do_bulk_tx(&self,
                        client_id: &str,
                        request: &TxBulkQueryRequest) -> Result<Vec<TxQuerySuccessResponse>, QueryError> {
        let connection_id: &str = &request.connection_id;

        if !self.access_control_service.is_allowed(client_id, connection_id)? {
            return Err(QueryError::ForbiddenConnectionId);
        }

        if request.queries.is_empty() {
            return Ok(Vec::new());
        }

        match self.connection_service.get(connection_id).await {
            Some(connection_result) => {
                let mut connection = connection_result?;

                let tx: Transaction = connection.transaction().await?;

                let mut query_futures = Vec::new();

                for (i, query_request_body) in request.queries.iter().enumerate() {
                    let query_future =
                        QueryService::do_execute_bulk(&tx, &query_request_body,i);

                    query_futures.push(query_future);
                }

                let results: Vec<Result<TxQuerySuccessResponse, QueryError>> =
                    future::join_all(query_futures).await;

                let mut successes = Vec::new();

                for result in results {
                    successes.push(result?);
                }

                tx.commit().await?;

                Ok(successes)
            },
            None => Err(QueryError::UnknownDatabaseConnection(connection_id.to_owned())),
        }
    }

    async fn do_execute_bulk(tx: &Transaction<'_>,
                             bulk_body: &TxBulkQueryRequestBody,
                             ord_num: usize) -> Result<TxQuerySuccessResponse, QueryError> {
        let stmt =
            tx.prepare_cached(&bulk_body.query).await?;

        let mut param_values_arena = Vec::new();

        for params_row in &bulk_body.params {
            let param_values: Vec<&(dyn ToSql + Sync)> = convert_params(
                stmt.params(),
                &params_row.values
            )?;

            param_values_arena.push(param_values);
        }

        let mut query_futures = Vec::new();

        for param_values in param_values_arena.as_slice() {
            let query_future =
                tx.query(&stmt, param_values);

            query_futures.push(query_future);
        }

        let query_results = future::join_all(query_futures).await;

         let mut results = Vec::new();

        for query_result in query_results {
            results.push(convert_rows(query_result?)?);
        }

        let column_names =
            results.first()
                .map_or(Vec::new(), |cwr| cwr.1.clone());

        let row_groups =
            results.into_iter()
                .map(|cwr| RowResponseGroup { rows: cwr.0 })
                .collect();

        Ok(TxQuerySuccessResponse {
            ord_num: ord_num as i32,
            column_names,
            row_groups
        })
    }

    async fn do_query(&self,
                      client_id: &str,
                      request: &QueryRequest) -> Result<QuerySuccessResponse, QueryError> {
        let connection_id: &str = &request.connection_id;

        if !self.access_control_service.is_allowed(client_id, connection_id)? {
            return Err(QueryError::ForbiddenConnectionId);
        }

        match self.connection_service.get(connection_id).await {
            Some(connection_result) => {
                let connection = connection_result?;

                let stmt = connection.prepare_cached(&request.query).await?;

                let params: Vec<&(dyn ToSql + Sync)> =
                    convert_params(stmt.params(), &request.params)?;

                let results =
                    connection.query(&stmt, params.as_slice()).await?;

                let cwr = convert_rows(results)?;

                Ok(
                    QuerySuccessResponse {
                        rows: cwr.0,
                        column_names: cwr.1
                    }
                )
            }
            None => Err(QueryError::UnknownDatabaseConnection(connection_id.to_owned()))
        }
    }

    fn build_response<T: From<ErrorResponse>>(err: QueryError,
                                              correlation_id: &str) -> ResponseWithCode<T> {
        let code = err.get_code();
        ResponseWithCode(err, code)
            .map(|err|
                err.to_error_response(correlation_id.to_string()))
            .map(|err_response| err_response.into())
    }
}

impl From<QuerySuccessResponse> for QueryResponse {
    fn from(success: QuerySuccessResponse) -> Self {
        QueryResponse {
            payload: Some(query_response::Payload::Success(success))
        }
    }
}

impl From<ErrorResponse> for QueryResponse {
    fn from(err: ErrorResponse) -> Self {
        QueryResponse {
            payload: Some(query_response::Payload::Error(err))
        }
    }
}

impl From<Vec<TxQuerySuccessResponse>> for TxBulkQueryResponse {
    fn from(responses: Vec<TxQuerySuccessResponse>) -> Self {
        TxBulkQueryResponse {
            payload: Some(tx_bulk_query_response::Payload::Success(
                TxBulkQuerySuccessResponse {
                    responses
                }))
        }
    }
}

impl From<ErrorResponse> for TxBulkQueryResponse {
    fn from(err: ErrorResponse) -> Self {
        TxBulkQueryResponse {
            payload: Some(tx_bulk_query_response::Payload::Error(err))
        }
    }
}


