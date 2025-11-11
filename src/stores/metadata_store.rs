use pgrx::{pg_schema, spi::Query, PgBuiltInOids, PgOid, Spi};

use crate::{
    types::{
        errors::SqlgenError,
        structs::table_metadata::{CrawlSchema, TableMetadata},
        traits::driver::TextEncoderDriver,
    },
    utils::sql::get_column_heap,
};

pub struct MetadataStore {}

impl MetadataStore {
    pub async fn initialize_metadata(
        engine: &str,
        schema: &str,
        encoder: Box<dyn TextEncoderDriver>,
    ) -> Result<(), SqlgenError> {
        let query = "SELECT sqlgen_internal.initialize_metadata($1, $2, $3);";
        let vector_size = i32::try_from(encoder.dimensions().await?)?;

        Spi::run_with_args(query, &[engine.into(), schema.into(), vector_size.into()])?;

        Self::populate_metadata_table(engine, schema, encoder).await?;

        Ok(())
    }

    pub async fn populate_metadata_table(
        engine: &str,
        schema: &str,
        encoder: Box<dyn TextEncoderDriver>,
    ) -> Result<(), SqlgenError> {
        let query =
            "SELECT table_name, column_name, ddl, comment FROM sqlgen_internal.crawl_schema($1);";

        let schemas: Result<Vec<CrawlSchema>, SqlgenError> = Spi::connect(|client| {
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

        Self::add_to_metadata_table(engine, metadatas)
    }

    pub fn add_to_metadata_table(
        engine: &str,
        metadatas: Vec<TableMetadata>,
    ) -> Result<(), SqlgenError> {
        let query = format!(
            "
            INSERT INTO sqlgen_internal.db_metadata_{} (
                schema_name, table_name, column_name, ddl, comment, ddl_vector, comment_vector
            ) VALUES ($1, $2, $3, $4, $5, $6::VECTOR, $7::VECTOR);
        ",
            engine
        );

        Spi::connect(|client| {
            let statement = &client.prepare(
                &query,
                &[
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::TEXTOID),
                    PgOid::from(PgBuiltInOids::FLOAT4ARRAYOID),
                    PgOid::from(PgBuiltInOids::FLOAT4ARRAYOID),
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

    pub fn remove_metadata(engine: &str, model_name: &str) -> Result<(), SqlgenError> {
        let query = "SELECT sqlgen_internal.remove_metadata($1, $2);";

        Spi::run_with_args(query, &[engine.into(), model_name.into()])?;

        Ok(())
    }

    pub fn get_similar_ddls(
        engine: &str,
        user_query: Vec<f32>,
        limit: i32,
    ) -> Result<Vec<String>, SqlgenError> {
        let query = "SELECT sqlgen_internal.get_similar_ddls($1, $2::VECTOR, $3) AS ddl;";

        Spi::connect(|client| {
            let rows = client.select(
                query,
                None,
                &[engine.into(), user_query.into(), limit.into()],
            )?;

            let mut results = Vec::new();

            for row in rows {
                let ddl: String = get_column_heap(&row, "ddl")?;

                results.push(ddl);
            }

            Ok(results)
        })
    }

    pub fn get_ddls(engine: &str) -> Result<Vec<String>, SqlgenError> {
        let query = "SELECT ddl FROM sqlgen_internal.get_ddls($1) AS t(ddl)";

        Spi::connect(|client| {
            let rows = client.select(query, None, &[engine.into()])?;

            let mut results = Vec::new();

            for row in rows {
                let ddl: String = get_column_heap(&row, "ddl")?;

                results.push(ddl);
            }

            Ok(results)
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::Spi;
    use tokio::runtime::Runtime;

    use crate::{
        pg_test,
        stores::{metadata_store::MetadataStore, model_store::ModelStore},
        utils::{
            sql::{get_column, get_column_heap, get_column_heap_optional},
            test_utils::{create_engine, create_schema},
        },
    };

    #[pg_test]
    fn test_init_schema() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name, "smart");
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = Runtime::new().unwrap();

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap()
        });

        let query = format!(
            r#"
            SELECT
                schema_name,
                comment,
                ddl_vector::TEXT as ddl_vector,
                comment_vector::TEXT as comment_vector
            FROM sqlgen_internal.db_metadata_{};
        "#,
            engine_name
        );

        Spi::connect(|client| {
            let rows = client.select(&query, None, &[]).unwrap();

            if rows.is_empty() {
                panic!("db metadata rows are empty")
            }

            let mut meta_count = 0;

            for row in rows {
                meta_count += 1;

                let written_schema_name: String = get_column_heap(&row, "schema_name").unwrap();
                let ddl_vector: String = get_column_heap(&row, "ddl_vector").unwrap();
                let comment: String = get_column_heap(&row, "comment").unwrap();
                let comment_vector: String = get_column_heap(&row, "comment_vector").unwrap();

                assert_eq!(written_schema_name, schema_name);
                assert_eq!(ddl_vector, "[0,0.1,0.2,0.3]");
                assert_eq!(comment, "Hello world");
                assert_eq!(comment_vector, "[0,0.1,0.2,0.3]");
            }

            assert_eq!(meta_count, 18);
        });

        // Create table
        let table_name = format!("{}.authors", engine.schema_name);
        Spi::run(
            format!(
                r#"
                    CREATE TABLE {} (
                        author_id SERIAL PRIMARY KEY,
                        name TEXT NOT NULL
                    ); 
                    "#,
                table_name
            )
            .as_str(),
        )
        .unwrap();

        let query = format!(
            r#"
                SELECT
                    schema_name,
                    table_name,
                    column_name,
                    ddl,
                    comment,
                    ddl_vector::TEXT as ddl_vector,
                    comment_vector::TEXT as comment_vector
                FROM sqlgen_internal.db_metadata_{}
                WHERE table_name = $1
                "#,
            engine_name
        );

        let mut count = 0;
        Spi::connect(|client| {
            let rows = client.select(&query, None, &["authors".into()]).unwrap();

            for row in rows {
                count += 1;

                let schema_name: String = get_column_heap(&row, "schema_name").unwrap();
                let table_name: String = get_column_heap(&row, "table_name").unwrap();
                let ddl_vector: String = get_column_heap(&row, "ddl_vector").unwrap();

                assert_eq!(schema_name, "test_example");
                assert_eq!(table_name, "authors");
                assert_eq!(ddl_vector, "[0,0.1,0.2,0.3]");
            }

            assert_eq!(count, 2)
        });

        // Alter table add column
        let table_name = format!("{}.authors", engine.schema_name);
        Spi::run(
            format!(
                r#"
                ALTER TABLE {} ADD COLUMN age INT;
                "#,
                table_name
            )
            .as_str(),
        )
        .unwrap();

        let query = format!(
            r#"
                SELECT
                    schema_name,
                    table_name,
                    column_name,
                    ddl,
                    comment,
                    ddl_vector::TEXT as ddl_vector,
                    comment_vector::TEXT as comment_vector
                FROM sqlgen_internal.db_metadata_{}
                WHERE table_name = 'authors'
                AND column_name = 'age';
                "#,
            engine_name
        );

        Spi::connect(|client| {
            let rows = client.select(&query, None, &[]).unwrap();
            let row = rows.first();

            let schema_name: String = get_column(&row, "schema_name").unwrap();
            let table_name: String = get_column(&row, "table_name").unwrap();
            let ddl_vector: String = get_column(&row, "ddl_vector").unwrap();

            assert_eq!(schema_name, "test_example");
            assert_eq!(table_name, "authors");
            assert_eq!(ddl_vector, "[0,0.1,0.2,0.3]");
        });

        // Delete table
        let table_name = format!("{}.authors", engine.schema_name);
        Spi::run(
            format!(
                r#"
                DROP TABLE {};
                "#,
                table_name
            )
            .as_str(),
        )
        .unwrap();

        let query = format!(
            r#"
                SELECT
                    count(*)::TEXT as row_count
                FROM sqlgen_internal.db_metadata_{}
                WHERE table_name = 'authors'
                "#,
            engine_name
        );

        Spi::connect(|client| {
            let rows = client.select(&query, None, &[]).unwrap();
            let row = rows.first();

            let row_count: String = get_column(&row, "row_count").unwrap();

            assert_eq!(row_count, "0");
        });

        // Add comment
        Spi::run(
            format!(
                r#"
                COMMENT ON COLUMN {schema}.order_items.quantity IS 'This is a test';
                COMMENT ON COLUMN {schema}.order_items.product_id IS NULL;
                "#,
                schema = engine.schema_name,
            )
            .as_str(),
        )
        .unwrap();

        let query = format!(
            r#"
                SELECT
                    column_name,
                    comment,
                    comment_vector::TEXT as comment_vector
                FROM sqlgen_internal.db_metadata_{}
                WHERE table_name = 'order_items'
                "#,
            engine_name
        );

        let mut count = 0;

        Spi::connect(|client| {
            let rows = client
                .select(&query, None, &["order_items".into()])
                .unwrap();

            for row in rows {
                count += 1;

                let column_name: String = get_column_heap(&row, "column_name").unwrap();
                let comment: Option<String> = get_column_heap_optional(&row, "comment").unwrap();
                let comment_vector: Option<String> =
                    get_column_heap_optional(&row, "comment_vector").unwrap();

                if column_name == "quantity" {
                    count += 1;
                    assert_eq!(comment.unwrap(), "This is a test");
                    assert_eq!(comment_vector.unwrap(), "[0,0.1,0.2,0.3]");
                } else if column_name == "product_id" {
                    count += 1;
                    assert!(comment.is_none());
                    assert!(comment_vector.is_none());
                }
            }
        });
    }

    #[pg_test]
    fn test_get_ddls() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name, "smart");
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = Runtime::new().unwrap();

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap()
        });

        let ddls = MetadataStore::get_ddls(engine_name).unwrap();

        assert_eq!(ddls.len(), 18);
    }

    #[pg_test]
    fn test_get_similar_ddls() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name, "smart");
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = Runtime::new().unwrap();
        let count = 10;

        let user_query = rt.block_on(async { encoder.encode("test").await.unwrap() });

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap()
        });

        let ddls = MetadataStore::get_similar_ddls(engine_name, user_query, count.clone()).unwrap();

        assert_eq!(ddls.len(), count as usize);
    }

    #[pg_test]
    fn test_remove_metadata() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name, "smart");
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = Runtime::new().unwrap();

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap()
        });

        MetadataStore::remove_metadata(engine_name, &engine.encoder_model).unwrap();

        assert!(MetadataStore::get_ddls(engine_name).is_err());
    }
}
