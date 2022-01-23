use actix_web::web::Query;
use bytes::BytesMut;
use deadpool::managed::PoolError;
use postgres_types::{IsNull, ToSql, Type};
use tokio_postgres::Error;

use crate::models::errors::QueryError;
use crate::models::parameters::convert_params;
use crate::models::payloads::{ErrorResponse, QueryRequest, QueryResponse, ValueWrapper};
use crate::models::payloads::error_response::ErrorType;
use crate::models::payloads::query_response::Payload;
use crate::models::payloads::QuerySuccessResponse;
use crate::models::payloads::value_wrapper::Value;
use crate::models::rows::convert_rows;
use crate::services::connections::{Connection, ConnectionService};

pub struct QueryService {

    connection_service: ConnectionService

}


impl QueryService {

    pub fn new(connection_service: ConnectionService) -> Self {
        QueryService {
            connection_service
        }
    }

    pub async fn query(&self,
                       request: &QueryRequest,
                       correlation_id: &str) -> QueryResponse {
        match self.do_query(request).await {
            Ok(ok) => ok.into(),
            Err(err) =>
                err.to_error_response(correlation_id.to_string()).into()
        }
    }

    async fn do_query(&self, request: &QueryRequest) -> Result<QuerySuccessResponse, QueryError> {
        let db_id: &str = &request.db_id;

        match self.connection_service.get(db_id).await {
            Some(connection_result) => {
                let connection = connection_result?;

                let stmt = connection.prepare_cached(&request.query).await?;

                let params: Vec<&(dyn ToSql + Sync)> =
                    convert_params(stmt.params(),&request.params)?;

                let results =
                    connection.query(&stmt, params.as_slice()).await?;

                convert_rows(results)
            }
            None => Err(QueryError::UnknownDatabaseConnection(db_id.to_owned()))
        }
    }
}

impl From<QuerySuccessResponse> for QueryResponse {
    fn from(success: QuerySuccessResponse) -> Self {
        QueryResponse {
            payload: Some(Payload::Success(success))
        }
    }
}

impl From<ErrorResponse> for QueryResponse {
    fn from(err: ErrorResponse) -> Self {
        QueryResponse {
            payload: Some(Payload::Error(err))
        }
    }
}
