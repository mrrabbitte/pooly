use std::error::Error;
use std::io::Read;

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

        let value = match col_type.oid() {
            16 => get_or_empty(&row, |val| Ok(Value::Bool(val)), i)?,
            17 => get_or_empty(&row, |val| Ok(Value::Bytes(val)), i)?,
            114 => get_or_empty(&row, proto_json, i)?,
            3802 => get_or_empty(&row, proto_json, i)?,
            25 => get_or_empty(&row, proto_string, i)?,
            1043 => get_or_empty(&row, proto_string, i)?,
            18 => get_or_empty(&row, proto_char, i)?,
            19 => get_or_empty(&row, |val| Ok(Value::String(val)), i)?,
            20 => get_or_empty(&row, |val| Ok(Value::Int8(val)), i)?,
            23 => get_or_empty(&row, |val| Ok(Value::Int4(val)), i)?,
            21 => get_or_empty(&row, |val| Ok(Value::Int4(val)), i)?,
            700 => get_or_empty(&row, |val| Ok(Value::Float(val)), i)?,
            701 => get_or_empty(&row, |val| Ok(Value::Double(val)), i)?,
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
        F: Fn(T) -> Result<Value, QueryError> {
    let value_maybe: Option<T> = row.try_get(i)?;

    match value_maybe {
        None => Ok(None),
        Some(value) => Ok(Some(constructor(value)?))
    }
}

fn proto_char(val: i8) -> Result<Value, QueryError> {
    Ok(Value::Char(val as i32))
}

fn proto_json(val: RawJsonBytes) -> Result<Value, QueryError> {
    let utf_json = String::from_utf8(val.bytes)?;

    Ok(Value::Json(utf_json.into()))
}

fn proto_string(val: String) -> Result<Value, QueryError> {
    Ok(Value::String(val.into()))
}

struct RawJsonBytes {
    bytes: Vec<u8>
}

impl<'a> FromSql<'a> for RawJsonBytes {
    fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        if !Self::accepts(ty) {
            return Err("The type cannot be treated as json raw bytes.".into());
        }

        if *ty == Type::JSONB {
            let mut b = [0; 1];
            raw.read_exact(&mut b)?;

            if b[0] != 1 {
                return Err("Unsupported JSONB encoding version".into());
            }
        }

        Ok(
            RawJsonBytes {
                bytes: raw.to_vec()
            }
        )
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::JSONB || *ty == Type::JSON
    }
}
