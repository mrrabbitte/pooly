use std::error::Error;

use postgres_types::{FromSql, Type};
use tokio_postgres::Row;

use crate::models::errors::QueryError;
use crate::models::payloads::{RowResponse, ValueWrapper};
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

    Ok(RowResponsesWithColumnNames(row_responses, column_names))
}

fn convert_row(row: &Row) -> Result<Vec<ValueWrapper>, QueryError> {
    let columns = row.columns();

    let mut values = vec![];

    for i in 0..columns.len() {
        let col_type = columns[i].type_();
///            Inner::Bool => 16,
//             Inner::Bytea => 17,
//             Inner::Char => 18,
//             Inner::Name => 19,
//             Inner::Int8 => 20,
//             Inner::Int2 => 21,
//             Inner::Int2Vector => 22,
//             Inner::Int4 => 23,
//             Inner::Regproc => 24,
//             Inner::Text => 25,
//             Inner::Oid => 26,
//             Inner::Tid => 27,
//             Inner::Xid => 28,
//             Inner::Cid => 29,
        let value = match col_type.oid() {
            25 => get_or_empty(&row, proto_string, i)?,
            1043 => get_or_empty(&row, proto_string, i)?,
            20 => get_or_empty(&row, Value::Int8, i)?,
            23 => get_or_empty(&row, Value::Int4, i)?,
            unknown => return Err(
                QueryError::UnknownPostgresValueType(
                    format!("Got unsupported row value type: {}, oid: {}.",
                            col_type.name(), unknown))
            )
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
