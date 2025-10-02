use pgrx::{spi::SpiTupleTable, FromDatum, IntoDatum};

use crate::types::errors::UtilError;

pub fn get_column<T: IntoDatum + FromDatum>(
    row: &SpiTupleTable,
    name: &str,
) -> Result<T, UtilError> {
    row.get_by_name::<T, _>(name)?
        .ok_or_else(|| UtilError::ColumnParseFailed(name.to_string()))
}
