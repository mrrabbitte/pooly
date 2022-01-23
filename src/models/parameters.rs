use postgres_types::{ToSql, Type};

use crate::models::errors::QueryError;
use crate::models::payloads::value_wrapper::Value;
use crate::models::payloads::ValueWrapper;

#[inline]
pub fn convert_params<'a>(expected_param_types: &[Type],
                          received_params: &'a Vec<ValueWrapper>)
    -> Result<Vec<&'a (dyn ToSql + Sync)>, QueryError> {

    if expected_param_types.len() != received_params.len() {
        return Err(
            QueryError::WrongNumParams(
                expected_param_types.len(),
                received_params.len())
        );
    }

    let mut params: Vec<&'a (dyn ToSql + Sync)> = vec![];

    for i in 0..expected_param_types.len() {
        let expected_type = &expected_param_types[i];
        let value_wrapper = &received_params[i];

        match &value_wrapper.value {
            None => params.push(get_null_for_type(expected_type)?),
            Some(Value::String(val)) => params.push(val),
            Some(Value::Int8(val)) => params.push(val),
            Some(Value::Bytes(val)) => params.push(val)
        }

    }

    Ok(params)
}

static NULL_STRING_VALUE: Option<String> = Option::None;
static NULL_INT8_VALUE: Option<i64> = Option::None;

#[inline]
fn get_null_for_type(c_type: &Type) -> Result<&'static (dyn ToSql + Sync), QueryError> {
    match c_type.oid() {
        25 => Ok(&NULL_STRING_VALUE),
        1043 => Ok(&NULL_STRING_VALUE),
        20 => Ok(&NULL_INT8_VALUE),
        _ => Err(QueryError::UnknownPostgresValueType(c_type.to_string()))
    }
}