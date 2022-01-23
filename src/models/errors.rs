use deadpool::managed::PoolError;
use tokio_postgres::Error;

use crate::models::payloads::{ErrorResponse, QueryResponse};
use crate::models::payloads::error_response::ErrorType;

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

    fn get_error_type(&self) -> ErrorType {
        match self {
            QueryError::UnknownDatabaseConnection(_) => ErrorType::MissingConnection,
            QueryError::PoolError(_) => ErrorType::PoolError,
            QueryError::PostgresError(_) => ErrorType::PostgresError,
            QueryError::WrongNumParams(_, _) => ErrorType::WrongNumOfParams,
            QueryError::UnknownPostgresValueType(_) => ErrorType::UnknownPgValueType
        }
    }

    fn get_message(self) -> String {
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
