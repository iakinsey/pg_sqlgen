use std::ffi::CString;

use uuid::Uuid;

use pgrx::{
    pg_sys::{panic::CaughtError, pg_parse_query, Node, NodeTag, RawStmt},
    spi::{SpiHeapTupleData, SpiTupleTable},
    FromDatum, IntoDatum, PgList, PgTryBuilder, Spi,
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
    format!("stmt{}", Uuid::new_v4().simple())
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

fn get_node_tags(query: &str) -> Result<Vec<NodeTag>, SqlgenError> {
    let cstr = CString::new(query)?;

    PgTryBuilder::new(|| unsafe {
        let raw_list = pg_parse_query(cstr.as_ptr());
        let pg_list = PgList::<RawStmt>::from_pg(raw_list);
        let mut tags = Vec::with_capacity(pg_list.len());

        for raw_stmt in pg_list.iter_ptr() {
            let node: *mut Node = (*raw_stmt).stmt;
            tags.push((*node).type_);
        }

        Ok(tags)
    })
    .catch_others(|e| Err(SqlgenError::UnsafeError(get_caught_error_string(e))))
    .execute()
}

pub fn can_validate_query_plan(query: &str) -> Result<bool, SqlgenError> {
    let tags = get_node_tags(query)?;

    if tags.len() != 1 {
        return Ok(false);
    }

    match tags[0] {
        NodeTag::T_SelectStmt
        | NodeTag::T_InsertStmt
        | NodeTag::T_UpdateStmt
        | NodeTag::T_DeleteStmt => Ok(true),
        _ => Ok(false),
    }
}

pub fn get_prepare_error(query: String) -> Result<Option<String>, SqlgenError> {
    match can_validate_query_plan(&query) {
        Err(e) => return Ok(Some(e.to_string())),
        Ok(false) => return Ok(None),
        Ok(true) => {}
    }

    let id = get_unique_prepared_statement_id();
    let prepare_query = format!("PREPARE {} AS {}", id, query);
    let prepare_query = match prepare_query.ends_with(";") {
        true => prepare_query.to_string(),
        false => format!("{};", prepare_query),
    };

    let explain_query = format!("EXPLAIN {}", query);
    let explain_query = match explain_query.ends_with(";") {
        true => explain_query.to_string(),
        false => format!("{};", explain_query),
    };

    let error: Option<String> = PgTryBuilder::new(|| {
        Spi::run(&prepare_query).unwrap();
        None
    })
    .catch_others(|e| Some(get_caught_error_string(e)))
    .catch_rust_panic(|e| Some(get_caught_error_string(e)))
    .execute();

    if error.is_some() {
        return Ok(error);
    }

    let error: Option<String> = PgTryBuilder::new(|| {
        Spi::run(&explain_query).unwrap();
        None
    })
    .catch_others(|e| Some(get_caught_error_string(e)))
    .catch_rust_panic(|e| Some(get_caught_error_string(e)))
    .execute();

    Ok(error)
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
