use pgrx::spi::SpiHeapTupleData;

use crate::{
    types::errors::ModelDriverError,
    utils::sql::{get_column_heap, get_column_heap_optional},
};

pub struct TableMetadata {
    pub schema_name: String,
    pub table_name: String,
    pub column_name: String,
    pub ddl: String,
    pub comment: Option<String>,
    pub ddl_vector: Vec<f32>,
    pub comment_vector: Option<Vec<f32>>,
}

pub struct CrawlSchema {
    pub table_name: String,
    pub column_name: String,
    pub ddl: String,
    pub comment: Option<String>,
}

impl CrawlSchema {
    pub fn from_row(row: SpiHeapTupleData) -> Result<Self, ModelDriverError> {
        let table_name: String = get_column_heap(&row, "table_name")?;
        let column_name: String = get_column_heap(&row, "column_name")?;
        let ddl: String = get_column_heap(&row, "ddl")?;
        let comment: Option<String> = get_column_heap_optional(&row, "comment")?;

        Ok(Self {
            table_name,
            column_name,
            ddl,
            comment,
        })
    }
}
