use bincode::ErrorKind;
use deadpool::managed::PoolError;
use deadpool_postgres::CreatePoolError;
use serde::{Deserialize, Serialize};
use tokio_postgres::Error;

use crate::models::payloads::{ErrorResponse, QueryResponse};
use crate::models::payloads::error_response::ErrorType;

#[derive(Debug)]
pub enum QueryError {

    ConnectionConfigError(String),
    CreatePoolError(String),
    UnknownDatabaseConnection(String),
    PoolError(String),
    PostgresError(String),
    WrongNumParams(usize, usize),
    UnknownPostgresValueType(String)

}

#[derive(Debug)]
pub enum ConnectionError {

    CreatePoolError(CreatePoolError),
    ConnectionConfigError(ConnectionConfigError),
    PoolError(PoolError<Error>),

}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionConfigError {

    ConfigStorageError(String),
    ConfigSerdeError(String),

}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecretsError {
    AlreadyInitialized,
    MasterKeyShareError(String),
    Unspecified,
    LockError,
    FileReadError
}

impl ConnectionConfigError {

    fn get_message(&self) -> &str {
        match self {
            ConnectionConfigError::ConfigStorageError(message) => message,
            ConnectionConfigError::ConfigSerdeError(message) => message
        }
    }

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
            QueryError::UnknownPostgresValueType(_) => ErrorType::UnknownPgValueType,
            QueryError::CreatePoolError(_) => ErrorType::CreatePoolError,
            QueryError::ConnectionConfigError(_) => ErrorType::ConnectionConfigError
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
                format!("Unknown pg type: {}", pg_type),
            QueryError::CreatePoolError(message) => message,
            QueryError::ConnectionConfigError(message) => message
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

impl From<ConnectionConfigError> for QueryError {
    fn from(err: ConnectionConfigError) -> Self {
       QueryError::ConnectionConfigError(err.get_message().to_string())
    }
}

impl From<ConnectionError> for QueryError {
    fn from(err: ConnectionError) -> Self {
        match err {
            ConnectionError::CreatePoolError(err) =>
                QueryError::CreatePoolError(err.to_string()),
            ConnectionError::PoolError(err) => err.into(),
            ConnectionError::ConnectionConfigError(err) => err.into()
        }
    }
}

impl From<Box<ErrorKind>> for ConnectionConfigError {
    fn from(err: Box<ErrorKind>) -> Self {
        ConnectionConfigError::ConfigSerdeError(err.to_string())
    }
}

impl From<sled::Error> for ConnectionConfigError {
    fn from(err: sled::Error) -> Self {
        ConnectionConfigError::ConfigStorageError(err.to_string())
    }
}