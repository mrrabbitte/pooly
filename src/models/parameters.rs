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
            Some(Value::Int4(val)) => params.push(val),
            Some(Value::Bytes(val)) => params.push(val)
        }

    }

    Ok(params)
}

const NULL_STRING_VALUE: Option<String> = Option::None;
const NULL_INT8_VALUE: Option<i64> = Option::None;

#[inline]
fn get_null_for_type(c_type: &Type) -> Result<&'static (dyn ToSql + Sync), QueryError> {
    match c_type.oid() {
        25 => Ok(&NULL_STRING_VALUE),
        1043 => Ok(&NULL_STRING_VALUE),
        20 => Ok(&NULL_INT8_VALUE),
        _ => Err(QueryError::UnknownPostgresValueType(c_type.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use postgres_types::Type;

    use crate::models::errors::QueryError;
    use crate::models::parameters::convert_params;
    use crate::models::payloads::value_wrapper::Value;
    use crate::models::payloads::ValueWrapper;

    #[test]
    fn test_converts_params_correctly() {
        let param_types = vec![
            Type::from_oid(25).unwrap(),
            Type::from_oid(1043).unwrap(),
            Type::from_oid(20).unwrap()
        ];

        let value_one = "some string value".to_string();
        let value_two = vec![2, 3, 1];
        let value_three = 123;

        let values = vec![
            ValueWrapper { value: Some(Value::String(value_one.clone())) },
            ValueWrapper { value: Some(Value::Bytes(value_two.clone())) },
            ValueWrapper { value: Some(Value::Int8(value_three.clone())) }
        ];

        let converted =
            convert_params(&param_types, &values).unwrap();

        assert_eq!(converted.len(), values.len());
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
