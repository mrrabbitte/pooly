use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::string::FromUtf8Error;
use std::sync::PoisonError;

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use bincode::ErrorKind;
use deadpool::managed::PoolError;
use deadpool_postgres::CreatePoolError;
use ring::error::Unspecified;
use serde::{Deserialize, Serialize};
use sled::CompareAndSwapError;
use sled::transaction::TransactionError;
use tokio_postgres::Error;

use crate::models::payloads::error_response::ErrorType;
use crate::models::payloads::ErrorResponse;

#[derive(Debug)]
pub enum QueryError {

    ConnectionConfigError(String, u16),
    CreatePoolError(String),
    ForbiddenConnectionId,
    UnknownDatabaseConnection(String),
    PoolError(String),
    PostgresError(String),
    StorageError,
    UnknownPostgresValueType(String),
    WrongNumParams(usize, usize),
    ReadUtfError(String)

}

#[derive(Debug)]
pub enum ConnectionError {

    CreatePoolError(CreatePoolError),
    ConnectionConfigError(ConnectionConfigError),
    PoolError(PoolError<Error>),
    StorageError(StorageError)

}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionConfigError {

    AlreadyExistsError,
    ConfigStorageError(StorageError),
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

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageError {

    AlreadyExistsError,
    CouldNotFindValueToUpdate,
    OptimisticLockingError {
        old_created_at: u128,
        new_created_at: u128,
        old_version: u32,
        new_version: u32
    },
    RetrievalError(String),
    SerdeError(String),
    SecretsError(SecretsError),
    SledError(String),
    TransactionError(String),
    Utf8Error

}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum WildcardPatternError {

    NoStars,
    TooManyStars,
    UnsupportedPattern

}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthError {

    InvalidClaims,
    InvalidHeader,
    InvalidToken,
    HmacError,
    NoneAlgorithmProvided,
    MissingAuthService,
    MissingAuthHeader,
    PemError,
    StorageError(StorageError),
    UnsupportedAlgorithm,
    UnknownKey,
    Forbidden,
    VerificationError(String),

}

#[derive(Debug, Serialize, Deserialize)]
pub enum InitializationError {

    AuthClearError,
    TooManyShares,
    SecretsError(SecretsError),
    StorageError(StorageError)

}

impl std::fmt::Display for WildcardPatternError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl ConnectionConfigError {

    fn get_message(&self) -> String {
        match self {
            ConnectionConfigError::AlreadyExistsError =>
                "Config for this connection id already exists".to_string(),
            ConnectionConfigError::ConfigStorageError(err) =>
                format!("{:?}", err),
            ConnectionConfigError::ConfigSerdeError(message) => message.clone(),
            ConnectionConfigError::SecretsError(message) => message.clone()
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
            QueryError::ForbiddenConnectionId => 403,
            QueryError::UnknownDatabaseConnection(_) => 400,
            QueryError::PoolError(_) => 500,
            QueryError::PostgresError(_) => 500,
            QueryError::WrongNumParams(_, _) => 400,
            QueryError::UnknownPostgresValueType(_) => 500,
            QueryError::StorageError => 500,
            QueryError::ReadUtfError(_) => 500
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
            QueryError::ConnectionConfigError(_, _) => ErrorType::ConnectionConfigError,
            QueryError::ForbiddenConnectionId => ErrorType::ForbiddenConnectionId,
            QueryError::StorageError => ErrorType::StorageError,
            QueryError::ReadUtfError(_) => ErrorType::PostgresError
        }
    }

    fn get_message(self) -> String {
        match self {
            QueryError::UnknownDatabaseConnection(missing_name) =>
                format!("Connection not found: {}", missing_name),
            QueryError::PoolError(message) => message,
            QueryError::PostgresError(message) => message,
            QueryError::WrongNumParams(actual, expected) =>
                format!("Expected: {} argument(s), got: {}", expected, actual),
            QueryError::UnknownPostgresValueType(pg_type) =>
                format!("Unknown pg type: {}", pg_type),
            QueryError::CreatePoolError(message) => message,
            QueryError::ConnectionConfigError(message, _) => message,
            QueryError::ForbiddenConnectionId =>
                "The connection of the requested id is forbidden.".into(),
            QueryError::StorageError =>
                "Underlying storage error.".into(),
            QueryError::ReadUtfError(message) => message
        }
    }

}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::PemError
            | AuthError::MissingAuthService
            | AuthError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            _ => StatusCode::UNAUTHORIZED
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(self)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&StatusCode::UNAUTHORIZED, f)
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
        let message = err.get_message();
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
            ConnectionError::ConnectionConfigError(err) => err.into(),
            ConnectionError::StorageError(err) => err.into()
        }
    }
}

impl From<Box<ErrorKind>> for ConnectionConfigError {
    fn from(err: Box<ErrorKind>) -> Self {
        ConnectionConfigError::ConfigSerdeError(err.to_string())
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

impl From<CompareAndSwapError> for StorageError {
    fn from(_: CompareAndSwapError) -> Self {
        StorageError::AlreadyExistsError
    }
}

impl<E: Debug> From<TransactionError<E>> for StorageError {
    fn from(err: TransactionError<E>) -> Self {
        StorageError::TransactionError(format!("{:?}", err))
    }
}

impl From<CompareAndSwapError> for ConnectionConfigError {
    fn from(_: CompareAndSwapError) -> Self {
        ConnectionConfigError::AlreadyExistsError
    }
}

impl From<sled::Error> for StorageError {
    fn from(err: sled::Error) -> Self {
        StorageError::SledError(err.to_string())
    }
}

impl From<Box<bincode::ErrorKind>> for StorageError {
    fn from(err: Box<ErrorKind>) -> Self {
        StorageError::SerdeError(format!("{:?}", err))
    }
}

impl From<SecretsError> for StorageError {
    fn from(err: SecretsError) -> Self {
        StorageError::SecretsError(err)
    }
}

impl From<FromUtf8Error> for StorageError {
    fn from(_: FromUtf8Error) -> Self {
        StorageError::Utf8Error
    }
}

impl From<StorageError> for ConnectionConfigError {
    fn from(err: StorageError) -> Self {
        ConnectionConfigError::ConfigStorageError(err)
    }
}

impl From<StorageError> for QueryError {
    fn from(_: StorageError) -> Self {
        QueryError::StorageError
    }
}

impl From<StorageError> for AuthError {
    fn from(err: StorageError) -> Self {
        AuthError::StorageError(err)
    }
}

impl From<FromUtf8Error> for QueryError {
    fn from(err: FromUtf8Error) -> Self {
        QueryError::ReadUtfError(format!("{:?}", err))
    }
}

impl From<SecretsError> for InitializationError {
    fn from(err: SecretsError) -> Self {
        InitializationError::SecretsError(err)
    }
}

impl From<StorageError> for InitializationError {
    fn from(err: StorageError) -> Self {
        InitializationError::StorageError(err)
    }
}

impl From<jwt::Error> for AuthError {
    fn from(err: jwt::Error) -> Self {
        AuthError::VerificationError(format!("{:?}", err))
    }
}
