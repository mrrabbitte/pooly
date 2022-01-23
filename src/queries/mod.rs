use actix_web::web::Query;
use bytes::BytesMut;
use deadpool::managed::PoolError;
use postgres_types::{IsNull, ToSql, Type};
use tokio_postgres::Error;

use crate::connections::{Connection, ConnectionService};
use crate::models::parameters::convert_params;
use crate::models::payloads::{ErrorResponse, QueryRequest, QueryResponse, ValueWrapper};
use crate::models::payloads::error_response::ErrorType;
use crate::models::payloads::query_response::Payload;
use crate::models::payloads::QuerySuccessResponse;
use crate::models::payloads::value_wrapper::Value;
use crate::models::rows::convert_rows;

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

    async fn do_query(&self, request: &QueryRequest)
        -> Result<QuerySuccessResponse, QueryError> {
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

#[derive(Debug)]
pub enum QueryError {
    UnknownDatabaseConnection(String),
    PoolError(String),
    PostgresError(String),
    WrongNumParams(usize, usize),
    UnknownPostgresValueType(String)
}

impl QueryError {

    pub fn to_error_response(self, correlation_id: String) -> ErrorResponse {
        let error_type = self.get_error_type();

        ErrorResponse {
            message: self.get_message(),
            error_type: error_type.into(),
            correlation_id
        }
    }

    pub fn get_error_type(&self) -> ErrorType {
        match self {
            QueryError::UnknownDatabaseConnection(_) => ErrorType::MissingConnection,
            QueryError::PoolError(_) => ErrorType::PoolError,
            QueryError::PostgresError(_) => ErrorType::PostgresError,
            QueryError::WrongNumParams(_, _) => ErrorType::WrongNumOfParams,
            QueryError::UnknownPostgresValueType(_) => ErrorType::UnknownPgValueType
        }
    }

    pub fn get_message(self) -> String {
        match self {
            QueryError::UnknownDatabaseConnection(missing_name) =>
               format!("Not found database: {}", missing_name),
            QueryError::PoolError(message) => message,
            QueryError::PostgresError(message) => message,
            QueryError::WrongNumParams(actual, expected) =>
                format!("Expected: {} argument(s), got: {}", expected, actual),
            QueryError::UnknownPostgresValueType(pg_type) =>
                format!("Unknown pg type: {}", pg_type)
        }
    }

}

impl From<PoolError<Error>> for QueryError {
    fn from(err: PoolError<Error>) -> Self {
        QueryError::PoolError(err.to_string())
    }
}

impl From<Error> for QueryError {
    fn from(err: Error) -> Self {
        QueryError::PostgresError(err.to_string())
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