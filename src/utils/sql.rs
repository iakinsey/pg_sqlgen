use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use pgrx::{
    pg_sys::Oid,
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

pub fn get_oid(name: &str) -> Result<PgOid, SqlgenError> {
    let mut map = OID_MAP.lock()?;
    let oid = map.get(name).copied();

    if oid.is_some() {
        return Ok(oid.unwrap());
    }

    let query = "SELECT oid::TEXT oid FROM pg_type WHERE typname = $1;";

    Spi::connect(|client| {
        let row = client.select(query, Some(1), &[name.into()])?;

        if row.is_empty() {
            return Err(SqlgenError::NotFound(name.to_string()));
        }

        let result: String = get_column(&row.first(), "oid")?;
        let int_val: u32 = result.parse()?;
        let oid = PgOid::from(Oid::from_u32(int_val));

        map.insert(name.to_string(), oid);

        Ok(oid)
    })
}

pub fn get_current_schema() -> Result<String, SqlgenError> {
    match Spi::get_one::<String>("SELECT current_schema()")? {
        Some(v) => Ok(v),
        None => Err(SqlgenError::NoSchema()),
    }
}
