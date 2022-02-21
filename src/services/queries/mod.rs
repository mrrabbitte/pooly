use std::sync::Arc;

use deadpool_postgres::Transaction;
use postgres_types::ToSql;

use crate::models::errors::QueryError;
use crate::models::parameters::convert_params;
use crate::models::payloads::{ErrorResponse, query_response, QueryRequest, QueryResponse, RowResponseGroup, tx_bulk_query_response, TxBulkQueryRequest, TxBulkQueryRequestBody, TxBulkQueryResponse, TxBulkQuerySuccessResponse, TxQuerySuccessResponse};
use crate::models::payloads::QuerySuccessResponse;
use crate::models::responses::ResponseWithCode;
use crate::models::rows::convert_rows;
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

        match self.connection_service.get(connection_id).await {
            Some(connection_result) => {
                let mut connection = connection_result?;

                let tx: Transaction = connection.transaction().await?;

                let mut results = Vec::new();

                for (i, query_request_body) in request.queries.iter().enumerate() {
                    let query_response =
                        QueryService::do_execute_bulk(&tx, &query_request_body,i)
                            .await?;

                    results.push(query_response);
                }

                tx.commit().await?;

                Ok(results)
            },
            None => Err(QueryError::UnknownDatabaseConnection(connection_id.to_owned())),
        }
    }

    async fn do_execute_bulk(tx: &Transaction<'_>,
                             bulk_body: &TxBulkQueryRequestBody,
                             ord_num: usize) -> Result<TxQuerySuccessResponse, QueryError> {
        let stmt =
            tx.prepare_cached(&bulk_body.query).await?;

        let mut results = Vec::new();

        for params_row in &bulk_body.params {
            let param_values: Vec<&(dyn ToSql + Sync)> = convert_params(
                stmt.params(),
                &params_row.values
            )?;

            let query_results =
                tx.query(&stmt, param_values.as_slice()).await?;

            results.push(convert_rows(query_results));
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

                let cwr = convert_rows(results);

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


