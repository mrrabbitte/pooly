use std::sync::PoisonError;

use bincode::ErrorKind;
use deadpool::managed::PoolError;
use deadpool_postgres::CreatePoolError;
use ring::error::Unspecified;
use serde::{Deserialize, Serialize};
use sled::CompareAndSwapError;
use tokio_postgres::Error;

use crate::models::payloads::error_response::ErrorType;
use crate::models::payloads::ErrorResponse;

#[derive(Debug)]
pub enum QueryError {

    ConnectionConfigError(String, u16),
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

    AlreadyExistsError,
    ConfigStorageError(String),
    ConfigSerdeError(String),
    SecretsError(String)

}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecretsError {
    AeadError(String),
    AlreadyInitialized,
    AlreadyUnsealed,
    FileReadError,
    LockError,
    MasterKeyShareError(String),
    Sealed,
    SerdeError(String),
    Unspecified
}

impl ConnectionConfigError {

    fn get_message(&self) -> &str {
        match self {
            ConnectionConfigError::AlreadyExistsError =>
                "Config for this connection id already exists",
            ConnectionConfigError::ConfigStorageError(message) => message,
            ConnectionConfigError::ConfigSerdeError(message) => message,
            ConnectionConfigError::SecretsError(message) => message
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

    pub fn get_code(&self) -> u16 {
        match self {
            QueryError::ConnectionConfigError(_, code) => *code,
            QueryError::CreatePoolError(_) => 500,
            QueryError::UnknownDatabaseConnection(_) => 400,
            QueryError::PoolError(_) => 500,
            QueryError::PostgresError(_) => 500,
            QueryError::WrongNumParams(_, _) => 400,
            QueryError::UnknownPostgresValueType(_) => 500
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
            QueryError::ConnectionConfigError(_, _) => ErrorType::ConnectionConfigError
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
            QueryError::ConnectionConfigError(message, _) => message
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
        let message = err.get_message().to_string();
        let code = match err {
            ConnectionConfigError::SecretsError(_) => 401,
            ConnectionConfigError::AlreadyExistsError => 409,
            _ => 500
        };
       QueryError::ConnectionConfigError(message, code)
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

impl From<Unspecified> for SecretsError {
    fn from(_: Unspecified) -> Self {
        SecretsError::Unspecified
    }
}

impl From<chacha20poly1305::aead::Error> for SecretsError {
    fn from(err: chacha20poly1305::aead::Error) -> Self {
        SecretsError::AeadError(format!("{:?}", err))
    }
}

impl From<SecretsError> for ConnectionConfigError {
    fn from(err: SecretsError) -> Self {
        ConnectionConfigError::SecretsError(format!("Secrets error: {:?}", err))
    }
}

impl<T> From<PoisonError<T>> for SecretsError {
    fn from(_: PoisonError<T>) -> Self {
        SecretsError::LockError
    }
}

impl From<Box<bincode::ErrorKind>> for SecretsError {
    fn from(err: Box<ErrorKind>) -> Self {
        SecretsError::SerdeError(format!("{:?}", err))
    }
}

impl From<CompareAndSwapError> for ConnectionConfigError {
    fn from(_: CompareAndSwapError) -> Self {
        ConnectionConfigError::AlreadyExistsError
    }
}