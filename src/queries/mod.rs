use actix_web::web::Query;
use bytes::BytesMut;
use deadpool::managed::PoolError;
use postgres_types::{IsNull, ToSql, Type};
use tokio_postgres::Error;

use crate::connections::{Connection, ConnectionService};
use crate::models::convert::convert;
use crate::models::payloads::{QueryRequest, ValueWrapper};
use crate::models::payloads::QueryResponse;
use crate::models::payloads::value_wrapper::Value;

pub struct QueryService {

    connection_service: ConnectionService

}

impl QueryService {

    pub fn new(connection_service: ConnectionService) -> Self {
        QueryService {
            connection_service
        }
    }

    pub async fn query(&self, request: &QueryRequest) -> Result<QueryResponse, QueryError> {
        let db_id: &str = &request.db_id;

        match self.connection_service.get(db_id).await {
            Some(connection_result) => {
                let connection = connection_result?;

                let mut params: Vec<&(dyn ToSql + Sync)> = vec![];

                for value_wrapper in &request.params {
                    match &value_wrapper.value {
                        None => {},
                        Some(val) => match val {
                            Value::String(v) => params.push(v),
                            Value::Int8(v) => params.push(v),
                            Value::Bytes(v) => params.push(v)
                        }
                    };
                }

                let results = connection.query(&request.query, &params).await?;

                convert(results).map_err(|err| QueryError::ConversionError)
            }
            None => Err(QueryError::MissingDatabaseConnection(db_id.to_owned()))
        }
    }

}

#[derive(Debug)]
pub enum QueryError {
    ConversionError,
    MissingDatabaseConnection(String),
    PoolError,
    PostgresError
}

impl From<PoolError<Error>> for QueryError {
    fn from(_: PoolError<Error>) -> Self {
        QueryError::PoolError
    }
}

impl From<Error> for QueryError {
    fn from(_: Error) -> Self {
        QueryError::PostgresError
    }
}