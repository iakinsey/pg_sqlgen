use uuid::Uuid;

use pgrx::{
    pg_sys::panic::CaughtError,
    spi::{SpiHeapTupleData, SpiTupleTable},
    FromDatum, IntoDatum, Spi,
};

use crate::types::errors::SqlgenError;

// Get column value from a query.
pub fn get_column<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<T, SqlgenError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| SqlgenError::ColumnParseFailed(name.to_string()))
}

// Get an optional column value from a query.
pub fn get_column_optional<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<Option<T>, SqlgenError> {
    Ok(row.get_by_name::<T, _>(name)?)
}

// Get column value from a query.
pub fn get_column_heap<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<T, SqlgenError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| SqlgenError::ColumnParseFailed(name.to_string()))
}

// Get an optional column value from a query.
pub fn get_column_heap_optional<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<Option<T>, SqlgenError> {
    Ok(row.get_by_name::<T, _>(name)?)
}

// Get the current Postgres schema in use.
pub fn get_current_schema() -> Result<String, SqlgenError> {
    match Spi::get_one::<String>("SELECT current_schema()::TEXT")? {
        Some(v) => Ok(v),
        None => Err(SqlgenError::NoSchema()),
    }
}

// Generates a guaranteed unique string to be used as a `PREPARE` identifier.
pub fn get_unique_prepared_statement_id() -> String {
    format!("stmt{}", Uuid::new_v4().simple().to_string())
}

pub fn get_caught_error_string(cause: CaughtError) -> String {
    match cause {
        CaughtError::PostgresError(e) | CaughtError::ErrorReport(e) => {
            let code = e.sql_error_code();
            let msg = e.message();

            format!("{:?}: {}", code, msg)
        }
        CaughtError::RustPanic {
            ereport,
            payload: _,
        } => ereport.message().to_string(),
    }
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

// These statements allow debug1 to be used in a testing enviornment
#[cfg(test)]
#[macro_export]
macro_rules! debug1 {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

#[cfg(not(test))]
#[allow(unused_imports)]
pub use pgrx::debug1;
