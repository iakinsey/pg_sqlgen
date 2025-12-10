use pgrx::Spi;

use crate::{
    stores::{
        config_store::{
            ConfigStore, VECTOR_COLUMN_IMPL_DEFAULT, VECTOR_COLUMN_IMPL_KEY,
            VECTOR_COLUMN_IMPL_PGVECTOR,
        },
        engine_store::EngineStore,
        metadata_store::MetadataStore,
    },
    types::errors::SqlgenError,
    utils::sql::get_column,
};

pub fn convert_vectors_to_pgvector() -> Result<(), SqlgenError> {
    if !has_vector_type()? {
        return Err(SqlgenError::UnsupportedScenario(
            "Database lacks VECTOR type",
        ));
    }

    let mut queries = vec![];

    for (engine, size) in MetadataStore::get_engines_and_vector_length()? {
        let query = format!(
            r#"
             ALTER TABLE sqlgen_internal.db_metadata_{engine}
                ALTER COLUMN ddl_vector
                    TYPE VECTOR({size})
                    USING (ddl_vector::VECTOR({size})),

            ALTER COLUMN comment_vector
                TYPE VECTOR({size})
                USING (comment_vector::VECTOR({size}));
        "#
        );

        queries.push(query)
    }

    let query = queries.join("");

    Spi::run(&query)?;
    ConfigStore::set_config_value(VECTOR_COLUMN_IMPL_KEY, VECTOR_COLUMN_IMPL_PGVECTOR)?;

    Ok(())
}

pub fn convert_vectors_to_in_memory() -> Result<(), SqlgenError> {
    let mut queries = vec![];

    for engine in EngineStore::list_engines()? {
        let query = format!(
            r#"
            ALTER TABLE sqlgen_internal.db_metadata_{engine}
                ALTER COLUMN ddl_vector
                    TYPE FLOAT4[]
                    USING ddl_vector::float4[],
                ALTER COLUMN comment_vector
                    TYPE FLOAT4[]
                    USING comment_vector::float4[];
        "#
        );

        queries.push(query)
    }

    let query = queries.join("");

    Spi::run(&query)?;
    ConfigStore::set_config_value(VECTOR_COLUMN_IMPL_KEY, VECTOR_COLUMN_IMPL_DEFAULT)?;

    Ok(())
}

pub fn has_vector_type() -> Result<bool, SqlgenError> {
    let query = r#"
        SELECT EXISTS (
            SELECT 1
            FROM pg_type
            WHERE typname = 'vector'
        ) AS has_vector; 
    "#;

    Spi::connect(|client| {
        let rows = client.select(query, None, &[])?;
        let row = rows.first();
        let exists: bool = get_column(&row, "has_vector").unwrap();

        Ok(exists)
    })
}

pub fn get_vector_column_type() -> String {
    match ConfigStore::get_config_value(VECTOR_COLUMN_IMPL_KEY) {
        Ok(s) if s == VECTOR_COLUMN_IMPL_PGVECTOR => "VECTOR".into(),
        Ok(s) if s == VECTOR_COLUMN_IMPL_DEFAULT => "FLOAT4[]".into(),
        Err(SqlgenError::ConfigDoesntExist(_)) => "FLOAT4[]".into(),
        _ => "FLOAT4[]".into(),
    }
}
