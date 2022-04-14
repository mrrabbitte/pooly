use postgres_types::{ToSql, Type};
use prost::Message;

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
            Some(Value::Bool(val)) => params.push(val),
            Some(Value::Bytes(val)) => params.push(val),
            Some(Value::Char(val)) => params.push(val),
            Some(Value::Double(val)) => params.push(val),
            Some(Value::Float(val)) => params.push(val),
            Some(Value::Int4(val)) => params.push(val),
            Some(Value::Int8(val)) => params.push(val),
            Some(Value::String(val)) => params.push(val),
            Some(Value::Json(val)) => params.push(val)
        }
    }

    Ok(params)
}

const NULL_BOOL_VALUE: Option<bool> = None;
const NULL_BYTES_VALUE: Option<Vec<u8>> = None;
const NULL_CHAR: Option<i8> = None;
const NULL_DOUBLE_VALUE: Option<f64> = None;
const NULL_FLOAT_VALUE: Option<f32> = None;
const NULL_STRING_VALUE: Option<String> = None;
const NULL_INT8_VALUE: Option<i64> = None;
const NULL_INT4_VALUE: Option<i32> = None;

#[inline]
fn get_null_for_type(c_type: &Type) -> Result<&'static (dyn ToSql + Sync), QueryError> {
    match c_type.oid() {
        25 => Ok(&NULL_STRING_VALUE),
        1043 => Ok(&NULL_STRING_VALUE),
        20 => Ok(&NULL_INT8_VALUE),
        16 => Ok(&NULL_BOOL_VALUE),
        17 => Ok(&NULL_BYTES_VALUE),
        114 => Ok(&NULL_STRING_VALUE),
        3802 => Ok(&NULL_STRING_VALUE),
        18 => Ok(&NULL_CHAR),
        19 => Ok(&NULL_STRING_VALUE),
        23 => Ok(&NULL_INT4_VALUE),
        21 => Ok(&NULL_INT4_VALUE),
        700 => Ok(&NULL_FLOAT_VALUE),
        701 => Ok(&NULL_DOUBLE_VALUE),
        _ => Err(QueryError::UnknownPostgresValueType(c_type.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use postgres_types::Type;

    use crate::models::errors::QueryError;
    use crate::models::payloads::value_wrapper::Value;
    use crate::models::payloads::ValueWrapper;
    use crate::models::query::parameters::convert_params;

    #[test]
    fn test_converts_params_correctly() {
        let param_types = vec![
            Type::from_oid(25).unwrap(),
            Type::from_oid(1043).unwrap(),
            Type::from_oid(20).unwrap(),
            Type::from_oid(16).unwrap(),
            Type::from_oid(17).unwrap(),
            Type::from_oid(114).unwrap(),
            Type::from_oid(3802).unwrap(),
            Type::from_oid(18).unwrap(),
            Type::from_oid(19).unwrap(),
            Type::from_oid(23).unwrap(),
            Type::from_oid(21).unwrap(),
            Type::from_oid(700).unwrap(),
            Type::from_oid(701).unwrap()
        ];

        let value_one = "some string value".to_string();
        let value_two = vec![2, 3, 1];
        let value_three = 123;
        let value_four = false;
        let value_five: Vec<u8> =
            vec![110, 101, 118, 101, 114, 103, 111, 110, 110, 97, 103, 105, 118, 101, 121, 111, 117, 117, 112];
        let value_six: i8 = 123;
        let value_seven: i32 = 321;

        let some_values = vec![
            ValueWrapper { value: Some(Value::String(value_one.clone())) },
            ValueWrapper { value: Some(Value::Bytes(value_two.clone())) },
            ValueWrapper { value: Some(Value::Int8(value_three.clone())) },
            ValueWrapper { value: Some(Value::Bool(value_four.clone())) },
            ValueWrapper { value: Some(Value::Bytes(value_five.clone())) },
            ValueWrapper { value: Some(Value::String(value_one.clone())) },
            ValueWrapper { value: Some(Value::String(value_one.clone())) },
            ValueWrapper { value: Some(Value::Char(value_six.clone() as i32)) },
            ValueWrapper { value: Some(Value::String(value_one.clone())) },
            ValueWrapper { value: Some(Value::Int4(value_seven.clone())) },
            ValueWrapper { value: Some(Value::Int4(value_seven.clone())) },
            ValueWrapper { value: Some(Value::Float(0.0321)) },
            ValueWrapper { value: Some(Value::Double(9.213)) },
        ];

        let none_values = vec![ValueWrapper { value: None }; 13];

        let mut converted =
            convert_params(&param_types, &some_values).unwrap();

        assert_eq!(converted.len(), some_values.len());

        converted =
            convert_params(&param_types, &none_values).unwrap();

        assert_eq!(converted.len(), some_values.len());
    }

    #[test]
    fn test_handles_wrong_num_params_correctly() {
        let param_types = vec![
            Type::from_oid(25).unwrap(),
            Type::from_oid(1043).unwrap(),
            Type::from_oid(20).unwrap()
        ];

        let values = vec![
            ValueWrapper { value: None }, ValueWrapper { value: None },
            ValueWrapper { value: None }, ValueWrapper { value: None }
        ];

        assert!(
            matches!(
                convert_params(&param_types, &values),
                Err(QueryError::WrongNumParams(_, _))
            )
        );
    }

    #[test]
    fn test_returns_query_error_on_unknown_param_type() {
        let unsupported =  Type::from_oid(601).unwrap();

        let mut param_types = vec![
            Type::from_oid(25).unwrap(),
            Type::from_oid(1043).unwrap(),
            Type::from_oid(20).unwrap()
        ];

        param_types.push(unsupported);

        let values = vec![
            ValueWrapper { value: None }, ValueWrapper { value: None },
            ValueWrapper { value: None }, ValueWrapper { value: None }
        ];

        assert!(
            matches!(
                convert_params(&param_types, &values),
                Err(QueryError::UnknownPostgresValueType(_))
            )
        );
    }
}
