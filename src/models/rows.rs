use std::error::Error;

use postgres_types::{FromSql, Type};
use tokio_postgres::Row;

use crate::models::payloads::{BytesWrapper, RowResponse};

pub struct RowResponsesWithColumnNames(pub Vec<RowResponse>, pub Vec<String>);

pub fn convert_rows(rows: Vec<Row>) -> RowResponsesWithColumnNames {
    let column_names = match rows.first() {
        None => Vec::default(),
        Some(row) =>
            row.columns().iter().map(|column| column.name().to_owned()).collect()
    };

    let mut row_responses = vec![];

    for row in &rows {
        row_responses.push(RowResponse{ values: convert_row(row)});
    }

    RowResponsesWithColumnNames(row_responses, column_names)
}

fn convert_row(row: &Row) -> Vec<BytesWrapper> {
    let mut values = vec![];

    for i in 0..row.len() {
        let value_maybe: Option<RawBytes> = row.get(i);

        values.push(BytesWrapper { value_maybe: value_maybe.map(RawBytes::unwrap) });
    }

    values
}

struct RawBytes {

    raw: Vec<u8>

}

impl<'a> RawBytes {

    fn unwrap(self) -> Vec<u8> {
        self.raw.to_vec()
    }

}

impl<'a> FromSql<'a> for RawBytes {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(
            RawBytes {
                raw: raw.to_vec()
            }
        )
    }

    fn accepts(_: &Type) -> bool {
        true
    }
}
