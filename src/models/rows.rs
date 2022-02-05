use std::any::Any;

use postgres_types::{FromSql, Oid};
use tokio_postgres::{Column, Error, Row};

use crate::models::errors::QueryError;
use crate::models::payloads::{QuerySuccessResponse, RowResponse, ValueWrapper};
use crate::models::payloads::value_wrapper::Value;

pub struct RowResponsesWithColumnNames(pub Vec<RowResponse>, pub Vec<String>);

pub fn convert_rows(rows: Vec<Row>) -> Result<RowResponsesWithColumnNames, QueryError> {
    let column_names = match rows.first() {
        None => Vec::default(),
        Some(row) =>
            row.columns().iter().map(|column| column.name().to_owned()).collect()
    };

    let mut row_responses = vec![];

    for row in &rows {
        row_responses.push(RowResponse{ values: convert_row(row)?});
    }

    Ok(
        RowResponsesWithColumnNames(row_responses, column_names)
    )
}

fn convert_row(row: &Row) -> Result<Vec<ValueWrapper>, QueryError> {
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
                          i: usize) -> Result<Option<Value>, QueryError>
    where
        T: FromSql<'a>,
        F: Fn(T) -> Value {
    let value_maybe: Option<T> = row.try_get(i)?;

    Ok(value_maybe.map(constructor))
}

fn proto_string(val: String) -> Value {
    Value::String(val.into())
}
