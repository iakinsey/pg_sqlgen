use pgrx::pg_extern;

use crate::stores::engine_store::EngineStore;
use crate::stores::model_store::ModelStore;
use crate::types::errors::SqlgenError;
use crate::utils::globals::get_runtime;
use pgrx::prelude::*;

// Allows internal SQL-defined functions to access text encoding models.
#[pg_extern]
fn internal_encode_text(engine: &str, text_value: &str) -> Result<Vec<f32>, SqlgenError> {
    let engine = EngineStore::get_engine(engine).unwrap();
    let profile = ModelStore::get_model_profile(&engine.encoder_model)?;
    let model = profile.get_text_encoder_model()?;
    let rt = get_runtime();

    rt.block_on(async { model.encode(text_value).await })
}

// Allows internal SQL-defined functions to access batch text encoding models.
#[pg_extern]
fn internal_batch_text_encode(
    model: &str,
    values: Vec<String>,
) -> Result<Vec<Vec<f32>>, SqlgenError> {
    let profile = ModelStore::get_model_profile(model)?;
    let model = profile.get_text_encoder_model()?;
    let rt = get_runtime();
    let values: Vec<&str> = values.iter().map(|s| s.as_str()).collect();

    rt.block_on(async { model.encode_many(&values).await })
}

// `sqlgen_internal` is a namespace restricted to internal sqlgen functions.
#[pg_schema]
mod sqlgen_internal {

    use std::collections::HashMap;

    use pgrx::pg_extern;
    use pgrx::prelude::*;
    use pgrx::spi::Query;

    use crate::drivers::get_model_descriptions;
    use crate::stores::certify_store::CertifyStore;
    use crate::stores::engine_store::EngineStore;
    use crate::stores::metadata_store::MetadataStore;
    use crate::stores::model_store::ModelStore;
    use crate::types::errors::SqlgenError;
    use crate::types::structs::engine::TableFilterType;
    use crate::types::structs::table_metadata::TableMetadata;
    use crate::utils::globals::get_runtime;
    use crate::utils::rpc::wrap_encode;
    use crate::utils::sql::get_column_heap;
    use crate::utils::sql::get_column_heap_optional;
    use crate::utils::sql::get_current_schema;

    // Create associated metadata rows for a table so that it can be tracked for
    // text-to-sql generation.
    #[pg_extern]
    fn add_table(engine_name: &str, schema: &str, table: &str) -> Result<(), SqlgenError> {
        let table_metadata_query = r#"
        SELECT
            a.attname::TEXT AS column_name,
            (
                format('%I %s', a.attname, format_type(a.atttypid, a.atttypmod))
                || CASE WHEN a.attidentity IN ('a','d') AND coalesce(a.attgenerated,'') = '' THEN
                    ' GENERATED ' || CASE a.attidentity WHEN 'a' THEN 'ALWAYS' ELSE 'BY DEFAULT' END || ' AS IDENTITY'
                ELSE '' END
                || CASE WHEN a.attgenerated = 's' THEN
                    ' GENERATED ALWAYS AS (' || pg_get_expr(ad.adbin, ad.adrelid) || ') STORED'
                ELSE '' END
                || CASE WHEN ad.adbin IS NOT NULL AND coalesce(a.attgenerated,'') = '' THEN
                    ' DEFAULT ' || pg_get_expr(ad.adbin, ad.adrelid)
                ELSE '' END
                || CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END
            ) AS ddl,
            pg_catalog.col_description(c.oid, a.attnum) AS comment
        FROM pg_attribute      a
        JOIN pg_class          c  ON c.oid = a.attrelid
        JOIN pg_namespace      n  ON n.oid = c.relnamespace
        LEFT JOIN pg_attrdef   ad ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
        WHERE n.nspname = $1
            AND c.relname = $2
            AND c.relkind IN ('r','p','v','m')
            AND a.attnum > 0
            AND NOT a.attisdropped
        ORDER BY a.attnum;
    "#;

        #[allow(clippy::type_complexity)]
        let (metadata, ddls, comments): (
            Vec<(String, String, Option<String>)>,
            Vec<&'static str>,
            Vec<(usize, &'static str)>,
        ) = Spi::connect(|client| {
            let mut m: Vec<(String, String, Option<String>)> = Vec::new();
            let mut d: Vec<&'static str> = Vec::new();
            let mut c: Vec<(usize, &'static str)> = Vec::new();

            let rows = client.select(table_metadata_query, None, &[schema.into(), table.into()])?;

            if rows.is_empty() {
                return Err(SqlgenError::TableDoesntExist(format!(
                    "{}.{}",
                    schema, table
                )));
            }

            for (i, row) in rows.enumerate() {
                let column_name: String = get_column_heap(&row, "column_name")?;
                let ddl: String = get_column_heap(&row, "ddl")?;
                let comment_opt: Option<String> = get_column_heap_optional(&row, "comment")?;

                m.push((column_name, ddl.clone(), comment_opt.clone()));

                d.push(Box::leak(ddl.into_boxed_str()));
                if let Some(com) = comment_opt {
                    c.push((i, Box::leak(com.into_boxed_str())));
                }
            }

            Ok((m, d, c))
        })?;

        let model_name = EngineStore::get_engine(engine_name)?.encoder_model;
        let model = ModelStore::get_model_profile(&model_name)?.get_text_encoder_model()?;
        let rt = get_runtime();

        #[allow(clippy::type_complexity)]
        let (ddl_encodings, comment_encodings): (
            Vec<Vec<f32>>,
            HashMap<usize, Option<Vec<f32>>>,
        ) = rt.block_on(async {
            let d = model.encode_many(&ddls).await?;
            let comment_strings: Vec<&str> = comments.iter().map(|(_, s)| *s).collect();
            let vectors = wrap_encode(model, comment_strings).await?;

            let mut map: HashMap<usize, Option<Vec<f32>>> = HashMap::new();
            for ((i, _), v) in comments.iter().zip(vectors) {
                map.insert(*i, v);
            }

            Ok::<(Vec<Vec<f32>>, HashMap<usize, Option<Vec<f32>>>), SqlgenError>((d, map))
        })?;

        let mut table_metadata: Vec<TableMetadata> = Vec::new();

        for (i, (column_name, ddl, comment)) in metadata.iter().enumerate() {
            let ddl_vector = ddl_encodings.get(i).cloned().ok_or_else(|| {
                SqlgenError::EncodingError(
                    engine_name.to_string(),
                    schema.to_string(),
                    table.to_string(),
                )
            })?;

            table_metadata.push(TableMetadata {
                schema_name: schema.to_string(),
                table_name: table.to_string(),
                column_name: column_name.clone(),
                ddl: ddl.clone(),
                comment: comment.clone(),
                ddl_vector,
                comment_vector: comment_encodings.get(&i).cloned().flatten(),
            })
        }

        let add_metadata_query = format!(
            r#"
        INSERT INTO sqlgen_internal.db_metadata_{} (
            schema_name,
            table_name,
            column_name,
            ddl,
            comment,
            ddl_vector,
            comment_vector
        ) VALUES ($1, $2, $3, $4, $5, $6::VECTOR, $7::VECTOR);
    "#,
            engine_name
        );

        Spi::connect(|client| {
            let statement = &client.prepare(
                &add_metadata_query,
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

            for metadata in table_metadata {
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

    // Lists model names and descriptions, for use by the
    // sqlgen.model_descriptions view.
    #[pg_extern]
    fn get_descriptions() -> TableIterator<
        'static,
        (
            name!(id, &'static str),
            name!(name, &'static str),
            name!(description, &'static str),
        ),
    > {
        TableIterator::new(get_model_descriptions())
    }

    // Creates a new text to sql engine.
    #[allow(clippy::too_many_arguments)]
    #[pg_extern]
    fn create_engine_external(
        name: &str,
        instruct_model: &str,
        encoder_model: &str,
        schema_name: Option<&str>,
        table_filter_type: Option<&str>,
        column_filter_limit: Option<i32>,
        error_correction_rounds: Option<i32>,
        system_prompt_template: Option<&str>,
        user_prompt_template: Option<&str>,
        relevant_ddls_template: Option<&str>,
        similar_queries_template: Option<&str>,
        filter_ddls_template: Option<&str>,
        syntax_correction_template: Option<&str>,
        explain_query_template: Option<&str>,
    ) -> Result<(), SqlgenError> {
        let schema_name = match schema_name {
            Some(s) => s.to_string(),
            None => get_current_schema()?,
        };

        let table_filter_type = match table_filter_type {
            Some(s) => TableFilterType::from_str(s)?,
            None => TableFilterType::Smart,
        };

        let column_filter_limit = column_filter_limit.unwrap_or(128);

        let error_correction_rounds = error_correction_rounds.unwrap_or(3);

        ModelStore::get_model_profile(instruct_model)?;
        let encoder = ModelStore::get_text_encoder_model(encoder_model)?;

        EngineStore::create_engine(
            name,
            &schema_name,
            encoder_model,
            instruct_model,
            table_filter_type.to_str(),
            column_filter_limit,
            error_correction_rounds,
            system_prompt_template,
            user_prompt_template,
            relevant_ddls_template,
            similar_queries_template,
            filter_ddls_template,
            syntax_correction_template,
            explain_query_template,
        )?;

        let rt = get_runtime();

        rt.block_on(async {
            let dims = encoder.dimensions().await?;
            CertifyStore::create_certified_queries_table(name, dims as i32)?;
            MetadataStore::initialize_metadata(name, &schema_name, encoder).await
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use pgrx::Spi;

    use crate::{drivers::get_model_descriptions, pg_test};

    #[pg_test]
    fn test_get_descriptions() {
        Spi::connect(|client| {
            let rows = client
                .select("SELECT * FROM sqlgen.model_descriptions;", None, &[])
                .unwrap();

            if rows.is_empty() {
                panic!("description rows are empty")
            }

            // Comparison seems a little silly here, we mostly just want to be sure
            // that the view is accessible.
            assert_eq!(get_model_descriptions().len(), rows.len());
        });
    }
}
