use pgrx::{
    spi::{SpiHeapTupleData, SpiTupleTable},
    FromDatum, IntoDatum,
};

use crate::types::errors::UtilError;

pub fn get_column<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<T, UtilError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| UtilError::ColumnParseFailed(name.to_string()))
}

pub fn get_column_optional<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<Option<T>, UtilError> {
    Ok(row.get_by_name::<T, _>(name)?)
}

pub fn get_column_heap<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<T, UtilError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| UtilError::ColumnParseFailed(name.to_string()))
}

pub fn get_column_heap_optional<T: IntoDatum + FromDatum>(
    row: &SpiHeapTupleData,
    name: &str,
) -> Result<Option<T>, UtilError> {
    Ok(row.get_by_name::<T, _>(name)?)
}
