use pgrx::{spi::Query, PgBuiltInOids, PgOid, Spi};

use crate::{
    types::{
        errors::StoreError,
        structs::table_metadata::{CrawlSchema, TableMetadata},
        traits::driver::TextEncoderDriver,
    },
    utils::sql::get_oid,
};

pub struct MetadataStore {}

impl MetadataStore {
    pub async fn initialize_metadata(
        model_name: &str,
        schema: &str,
        encoder: Box<dyn TextEncoderDriver>,
    ) -> Result<(), StoreError> {
        let query = "SELECT sqlgen_internal.initialize_metadata($1, $2, $3);";
        let vector_size = i32::try_from(encoder.dimensions().await?)?;

        Spi::run_with_args(
            query,
            &[model_name.into(), schema.into(), vector_size.into()],
        )?;

        Self::populate_metadata_table(model_name, schema, encoder).await?;

        Ok(())
    }

    pub async fn populate_metadata_table(
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

        Self::add_to_metadata_table(model_name, schema, metadatas)
    }

    pub fn add_to_metadata_table(
        model_name: &str,
        schema: &str,
        metadatas: Vec<TableMetadata>,
    ) -> Result<(), StoreError> {
        let query = format!(
            "
            INSERT INTO sqlgen_internal.db_metadata_{} (
                schema_name, table_name, column_name, ddl, comment, ddl_vector, comment_vector
            ) VALUES ($1, $2, $3, $4, $5, $6, $7);
        ",
            Self::get_unique_name(model_name, schema)
        );

        Spi::connect(|client| {
            let vector_oid = get_oid("vector")?;
            let statement = &client.prepare(
                &query,
                &[
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    vector_oid,
                    vector_oid,
                ],
            )?;

            for metadata in metadatas {
                statement.execute(
                    client,
                    None,
                    &[
                        metadata.schema_name.into(),
                        metadata.table_name.into(),
                        metadata.column_name.into(),
                        metadata.ddl.into(),
                        metadata.comment.into(),
                        metadata.ddl_vector.into(),
                        metadata.comment_vector.into(),
                    ],
                )?;
            }

            Ok(())
        })
    }

    pub fn remove_metadata(model_name: &str, schema: &str) -> Result<(), StoreError> {
        let query = "SELECT sqlgen_internal.remove_metadata($1, $2);";

        Spi::run_with_args(query, &[model_name.into(), schema.into()])?;

        Ok(())
    }

    // TODO querying metadata tables

    fn get_unique_name(model_name: &str, schema: &str) -> String {
        format!("{}_{}", schema, model_name)
    }
}
