use std::any::Any;

use postgres_types::{FromSql, Oid};
use tokio_postgres::{Error, Row};

use crate::values::payloads::ValueWrapper;
use crate::values::payloads::value_wrapper::Value;

pub fn convert(row: &Row) -> Result<Vec<ValueWrapper>, ConversionError> {
    let columns = row.columns();

    let mut values = vec![];

    for i in 0..columns.len() {
        let value = match columns[i].type_().oid() {
            25 => get_or_empty(&row, proto_string, i)?,
            1043 => get_or_empty(&row, proto_string, i)?,
            20 => get_or_empty(&row, Value::Int8, i)?,
            _ => panic!("Unknown value type.")
        };

        values.push(ValueWrapper { value });
    }

    Ok(values)
}

fn get_or_empty<'a, T, F>(row: &'a Row,
                          constructor: F,
                          i: usize) -> Result<Option<Value>, ConversionError>
    where
        T: FromSql<'a>,
        F: Fn(T) -> Value {
    let value_maybe: Option<T> = row.try_get(i)?;

    Ok(value_maybe.map(constructor))
}

fn proto_string(val: String) -> Value {
    Value::String(val.into())
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum ConversionError {
    PostgresError
}

impl From<Error> for ConversionError {
    fn from(_: Error) -> Self {
        ConversionError::PostgresError
    }
}



