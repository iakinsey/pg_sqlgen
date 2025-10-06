use pgrx::Spi;

use crate::types::{
    errors::{ModelDriverError, StoreError},
    structs::table_metadata::{CrawlSchema, TableMetadata},
    traits::driver::TextEncoderDriver,
};

pub struct MetadataStore {}

impl MetadataStore {
    pub fn install_metadata_table(model_name: &str, schema: &str, vector_size: usize) {
        unimplemented!()
    }

    pub async fn initialize_metadata_table(
        model_name: &str,
        schema: &str,
        encoder: Box<dyn TextEncoderDriver>,
    ) -> Result<(), StoreError> {
        let query =
            "SELECT table_name, column_name, ddl, comment FROM sqlgen_internal.crawl_schema($1);";

        let schemas: Result<Vec<CrawlSchema>, StoreError> = Spi::connect(|client| {
            let mut schemas: Vec<CrawlSchema> = Vec::new();
            let rows = client.select(query, None, &[schema.into()])?;

            for row in rows {
                let crawl_schema = CrawlSchema::from_row(row)?;

                schemas.push(crawl_schema);
            }

            Ok(schemas)
        });

        let schemas = schemas?;
        let ddls: Vec<&str> = schemas.iter().map(|c| c.ddl.as_str()).collect();
        let comments: Vec<String> = schemas
            .iter()
            .map(|c| c.comment.clone().unwrap_or_default())
            .collect();
        let comments: Vec<&str> = comments.iter().map(|s| s.as_str()).collect();
        let ddl_vecs = encoder.encode_many(&ddls).await?;
        let comment_vecs = encoder.encode_many(&comments).await?;
        let metadatas: Vec<TableMetadata> = schemas
            .into_iter()
            .zip(ddl_vecs)
            .zip(comment_vecs)
            .map(|((crawl_schema, ddl_vec), comment_vec)| TableMetadata {
                schema_name: schema.to_string(),
                table_name: crawl_schema.table_name,
                column_name: crawl_schema.column_name,
                ddl: crawl_schema.ddl,
                comment: crawl_schema.comment.clone(),
                ddl_vector: ddl_vec,
                comment_vector: if crawl_schema.comment.is_none() {
                    None
                } else {
                    Some(comment_vec)
                },
            })
            .collect();

        unimplemented!()
    }

    pub fn add_to_metadata_table(model_name: &str, schema: &str, metadatas: Vec<TableMetadata>) {
        let query = format!(
            "
            INSERT INTO sqlgen_internal.db_metadata_{} (
                schema_name, table_name, column_name, ddl, comment, ddl_vector, comment_vector
            ) VALUES ($1, $2, $3, $4, $5, $6, $7);
        ",
            schema
        );
        Spi::connect(|client| {
            let statement = client.prepare(&query, &[]);
            //client.prepare(&query)
            unimplemented!()
        });
    }

    pub fn remove_metadata_table(model_name: &str, schema: &str) {
        unimplemented!()
    }
}
