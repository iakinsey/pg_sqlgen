use std::collections::HashMap;

use pgrx::pg_extern;
use pgrx::prelude::*;
use pgrx::spi::Query;
use tokio::runtime::Runtime;

use crate::stores::engine_store::EngineStore;
use crate::stores::model_store::ModelStore;
use crate::types::errors::SqlgenError;
use crate::types::structs::table_metadata::TableMetadata;
use crate::utils::sql::get_column_heap;
use crate::utils::sql::get_column_heap_optional;

// These functions live in sqlgen_internal

#[pg_extern]
fn internal_encode_text(engine: &str, text_value: &str) -> Vec<f32> {
    let engine = EngineStore::get_engine(engine).unwrap();
    let profile =
        ModelStore::get_model_profile(&engine.encoder_model).unwrap_or_else(|e| error!("{}", e));
    let model = profile
        .get_text_encoder_model()
        .unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async { model.encode(text_value).await })
        .unwrap_or_else(|e| error!("{}", e))
}

#[pg_extern]
fn internal_batch_text_encode(model: &str, values: Vec<String>) -> Vec<Vec<f32>> {
    let profile = ModelStore::get_model_profile(model).unwrap_or_else(|e| error!("{}", e));
    let model = profile
        .get_text_encoder_model()
        .unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));
    let values: Vec<&str> = values.iter().map(|s| s.as_str()).collect();

    rt.block_on(async { model.encode_many(&values).await })
        .unwrap_or_else(|e| error!("{}", e))
}

#[pg_extern]
fn internal_add_table(engine: &str, schema_name: &str, table_name: &str) {
    _internal_add_table(engine, schema_name, table_name).unwrap_or_else(|e| error!("{}", e));
}

fn _internal_add_table(engine_name: &str, schema: &str, table: &str) -> Result<(), SqlgenError> {
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

            // For metadata (owned)
            m.push((column_name, ddl.clone(), comment_opt.clone()));

            // Leak to produce &'static str
            d.push(Box::leak(ddl.into_boxed_str()));
            if let Some(com) = comment_opt {
                c.push((i, Box::leak(com.into_boxed_str())));
            }
        }

        Ok((m, d, c))
    })?;

    let model_name = EngineStore::get_engine(engine_name)?.encoder_model;
    let model = ModelStore::get_model_profile(&model_name)?.get_text_encoder_model()?;
    let rt = Runtime::new()?;
    let (ddl_encodings, comment_encodings): (Vec<Vec<f32>>, HashMap<usize, Vec<f32>>) = rt
        .block_on(async {
            let d = model.encode_many(&ddls).await?;
            let comment_strings: Vec<&str> = comments.iter().map(|(_, s)| *s).collect();
            let vectors = model.encode_many(&comment_strings).await?;

            let mut map: HashMap<usize, Vec<f32>> = HashMap::new();
            for ((i, _), v) in comments.iter().zip(vectors) {
                map.insert(*i, v);
            }

            Ok::<(Vec<Vec<f32>>, HashMap<usize, Vec<f32>>), SqlgenError>((d, map))
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
            ddl_vector: ddl_vector,
            comment_vector: comment_encodings.get(&i).cloned(),
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

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use crate::pg_test;

    #[pg_test]
    fn test_add_table() {
        unimplemented!()
    }
}
