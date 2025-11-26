use pgrx::spi::SpiHeapTupleData;

use crate::{
    types::errors::SqlgenError,
    utils::sql::{get_column_heap, get_column_heap_optional},
};

// Metadata contains extension-readable information about a Postgres schema.
// This information is used in tandem with engines to provide runners with
// data necessary for execution.
pub struct TableMetadata {
    pub schema_name: String,
    pub table_name: String,
    pub column_name: String,
    pub ddl: String,
    pub comment: Option<String>,
    pub ddl_vector: Vec<f32>,
    pub comment_vector: Option<Vec<f32>>,
}

// The return type from the `sqlgen_internal.crawl_schema` SQL function. Used to
// populate TableMetadata.
pub struct CrawlSchema {
    pub table_name: String,
    pub column_name: String,
    pub ddl: String,
    pub comment: Option<String>,
}

impl CrawlSchema {
    pub fn from_row(row: SpiHeapTupleData) -> Result<Self, SqlgenError> {
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
