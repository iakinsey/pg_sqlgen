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

    pub fn remove_metadata(
        engine: &str,
        model_name: &str,
        schema: &str,
    ) -> Result<(), SqlgenError> {
        let query = "SELECT sqlgen_internal.remove_metadata($1, $2, $3);";

        Spi::run_with_args(query, &[engine.into(), model_name.into(), schema.into()])?;

        Ok(())
    }

    pub fn get_similar_ddls(
        engine: &str,
        user_query: Vec<f32>,
        limit: i32,
    ) -> Result<Vec<String>, SqlgenError> {
        let query = "SELECT sqlgen_internal.get_similar_ddls($1, $2, $3::REAL[]::VECTOR) AS ddl;";

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
        let query = "SELECT sqlgen_internal.get_ddls($1)";

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
    use serde_json::to_string;
    use tokio::runtime::Runtime;

    use crate::{
        pg_test,
        stores::{
            engine_store::EngineStore, metadata_store::MetadataStore, model_store::ModelStore,
        },
        types::structs::{
            engine::TextToSqlEngine, model_profile::ModelConfig, profiles::StubConfig,
        },
        utils::sql::{get_column, get_column_heap, get_column_heap_optional},
    };

    fn create_engine(name: &str, schema_name: &str) -> TextToSqlEngine {
        let table_filter_type = "smart";
        let encoder_model_name = "test_encoder_model_name";
        let instruct_model_name = "test_instruct_model_name";
        let expected_instruct_output = "expected_instruct_output";
        let expected_encoding_output = vec![0.0, 0.1, 0.2, 0.3];
        let config = StubConfig {
            instruct_output: expected_instruct_output.to_string(),
            encode_output: expected_encoding_output,
        };
        let encoder_profile = ModelConfig::Stub(config.clone());
        let instruct_profile = ModelConfig::Stub(config);
        let encoder_profile_json = to_string(&encoder_profile).unwrap();
        let instruct_profile_json = to_string(&instruct_profile).unwrap();

        Spi::run_with_args(
            "SELECT sqlgen.create_model($1, $2::JSONB);",
            &[encoder_model_name.into(), encoder_profile_json.into()],
        )
        .unwrap();

        Spi::run_with_args(
            "SELECT sqlgen.create_model($1, $2::JSONB);",
            &[instruct_model_name.into(), instruct_profile_json.into()],
        )
        .unwrap();

        EngineStore::create_engine(
            name,
            schema_name,
            encoder_model_name,
            instruct_model_name,
            table_filter_type,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        EngineStore::get_engine(name).unwrap()
    }

    fn create_schema(name: &str) {
        Spi::run(
            format!(
                r#"
                CREATE SCHEMA {name};

                CREATE TABLE {name}.users (
                    user_id SERIAL PRIMARY KEY,
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMP DEFAULT NOW()
                );

                COMMENT ON COLUMN {name}.users.user_id IS 'Hello world';
                COMMENT ON COLUMN {name}.users.username IS 'Hello world';
                COMMENT ON COLUMN {name}.users.email IS 'Hello world';
                COMMENT ON COLUMN {name}.users.created_at IS 'Hello world';


                CREATE TABLE {name}.products (
                    product_id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    price NUMERIC(10, 2) NOT NULL,
                    created_at TIMESTAMP DEFAULT NOW()
                );

                COMMENT ON COLUMN {name}.products.product_id IS 'Hello world';
                COMMENT ON COLUMN {name}.products.name IS 'Hello world';
                COMMENT ON COLUMN {name}.products.description IS 'Hello world';
                COMMENT ON COLUMN {name}.products.price IS 'Hello world';
                COMMENT ON COLUMN {name}.products.created_at IS 'Hello world';


                CREATE TABLE {name}.orders (
                    order_id SERIAL PRIMARY KEY,
                    user_id INT NOT NULL REFERENCES {name}.users(user_id) ON DELETE CASCADE,
                    order_date TIMESTAMP DEFAULT NOW(),
                    total NUMERIC(10, 2) NOT NULL
                );

                COMMENT ON COLUMN {name}.orders.order_id IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.user_id IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.order_date IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.total IS 'Hello world';


                CREATE TABLE {name}.order_items (
                    order_item_id SERIAL PRIMARY KEY,
                    order_id INT NOT NULL REFERENCES {name}.orders(order_id) ON DELETE CASCADE,
                    product_id INT NOT NULL REFERENCES {name}.products(product_id),
                    quantity INT NOT NULL CHECK (quantity > 0),
                    price NUMERIC(10, 2) NOT NULL
                );

                COMMENT ON COLUMN {name}.order_items.order_item_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.order_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.product_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.quantity IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.price IS 'Hello world';
            "#,
                name = name
            )
            .as_str(),
        )
        .unwrap();
    }

    #[pg_test]
    fn test_init_schema() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name);
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
        let table_name = format!("{}.order_items", engine.schema_name);
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
                    // TODO start here instead, figure out why comments arent updating
                    /*} else if column_name == "product_id" {
                        count += 1;
                        assert!(comment.is_none());
                        assert!(comment_vector.is_none());
                    */
                }
            }

            assert_eq!(count, 2)
        });
    }

    #[pg_test]
    fn test_get_ddls() {
        // TODO test
        unimplemented!()
    }

    #[pg_test]
    fn test_get_similar_ddls() {
        // TODO test
        unimplemented!()
    }

    #[pg_test]
    fn test_remove_metadata() {
        // TODO test
        unimplemented!()
    }
}
