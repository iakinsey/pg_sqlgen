use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

use lazy_static::lazy_static;
use pgrx::{
    spi::{SpiHeapTupleData, SpiTupleTable},
    FromDatum, IntoDatum, PgOid, Spi,
};

use crate::types::errors::SqlgenError;

pub fn get_column<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<T, SqlgenError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| SqlgenError::ColumnParseFailed(name.to_string()))
}

pub fn get_column_optional<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<Option<T>, SqlgenError> {
    Ok(row.get_by_name::<T, _>(name)?)
}

pub fn get_column_heap<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<T, SqlgenError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| SqlgenError::ColumnParseFailed(name.to_string()))
}

pub fn get_column_heap_optional<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<Option<T>, SqlgenError> {
    Ok(row.get_by_name::<T, _>(name)?)
}

lazy_static! {
    pub static ref OID_MAP: Mutex<HashMap<String, PgOid>> = Mutex::new(HashMap::new());
}

pub fn get_current_schema() -> Result<String, SqlgenError> {
    match Spi::get_one::<String>("SELECT current_schema()::TEXT")? {
        Some(v) => Ok(v),
        None => Err(SqlgenError::NoSchema()),
    }
}

pub fn get_unique_prepared_statement_id() -> String {
    format!("stmt{}", Uuid::new_v4().simple().to_string())
}

#[cfg(test)]
mod tests {
    use crate::utils::sql::get_unique_prepared_statement_id;

    #[test]
    fn test_get_assistant_response() {
        let id = get_unique_prepared_statement_id();

        assert!(!id.contains("-"));
        assert!(id.starts_with("stmt"));
    }
}
