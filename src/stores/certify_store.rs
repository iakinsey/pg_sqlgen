use pgrx::Spi;
use uuid::Uuid;

use crate::{types::errors::SqlgenError, utils::sql::get_column_heap};

pub struct CertifyStore {}

impl CertifyStore {
    pub fn create_certified_queries_table(
        engine: &str,
        vector_size: i32,
    ) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.create_certified_queries_table($1, $2);",
            &[engine.into(), vector_size.into()],
        )?;

        Ok(())
    }

    pub fn delete_certified_queries_table(engine: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.delete_certified_queries_table($1);",
            &[engine.into()],
        )?;

        Ok(())
    }

    pub fn certify_query(
        engine_name: &str,
        language_query: &str,
        sql_query: &str,
        language_vector: Vec<f32>,
    ) -> Result<String, SqlgenError> {
        let id = Uuid::new_v4().to_string();
        Spi::run_with_args(
            "SELECT sqlgen_internal.certify_query($1, $2::UUID, $3, $4, $5::VECTOR);",
            &[
                engine_name.into(),
                id.clone().into(),
                language_query.into(),
                sql_query.into(),
                language_vector.into(),
            ],
        )?;

        Ok(id)
    }

    pub fn get_certified_queries(
        engine_name: &str,
        language_query_vector: Vec<f32>,
        query_limit: i32,
    ) -> Result<Vec<(String, String)>, SqlgenError> {
        let query = r#"
            SELECT
                language_query,
                sql_query
            FROM sqlgen_internal.get_certified_queries($1, $2::VECTOR, $3)
            t(language_query, sql_query) 
            "#;

        Spi::connect(|client| {
            let rows = client.select(
                query,
                None,
                &[
                    engine_name.into(),
                    language_query_vector.into(),
                    query_limit.into(),
                ],
            )?;

            let mut results = Vec::new();

            for row in rows {
                let language_query: String = get_column_heap(&row, "language_query")?;
                let sql_query: String = get_column_heap(&row, "sql_query")?;

                results.push((language_query, sql_query))
            }

            Ok(results)
        })
    }

    pub fn decertify_query(engine: &str, id: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.decertify_query($1, $2::UUID);",
            &[engine.into(), id.into()],
        )?;

        Ok(())
    }
}
#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::{pg_test, stores::certify_store::CertifyStore};

    #[pg_test]
    fn test_certify_query_crud() {
        let engine = "test_engine";
        let vector_size = 4;
        let examples = vec![
            ("language query 1", "sql query 1", vec![0.0, 0.0, 0.0, 0.0]),
            ("language query 2", "sql query 2", vec![0.0, 0.0, 0.0, 1.1]),
            ("language query 3", "sql query 3", vec![0.0, 0.0, 1.1, 1.1]),
            ("language query 4", "sql query 4", vec![2.2, 2.2, 2.2, 2.2]),
        ];
        let mut ids = vec![];
        let should_be_closer = vec![2.2, 2.2, 2.2, 2.2];

        // Create tables
        CertifyStore::create_certified_queries_table(&engine, vector_size).unwrap();

        // Set certified queries
        for (language_query, sql_query, language_vector) in examples {
            let id =
                CertifyStore::certify_query(&engine, language_query, sql_query, language_vector)
                    .unwrap();

            ids.push(id);
        }

        // Get certified queries
        let queries =
            CertifyStore::get_certified_queries(&engine, should_be_closer.clone(), vector_size)
                .unwrap();

        assert_eq!(ids.len(), queries.len());
        let (first_lang, first_sql) = queries.get(0).unwrap();
        let (last_lang, last_sql) = queries.get(3).unwrap();

        assert_eq!(first_lang, "language query 4");
        assert_eq!(first_sql, "sql query 4");

        assert_eq!(last_lang, "language query 1");
        assert_eq!(last_sql, "sql query 1");

        // Delete a query
        let id = ids.get(3).unwrap();
        CertifyStore::decertify_query(&engine, id).unwrap();

        // Get certified queries
        let queries =
            CertifyStore::get_certified_queries(&engine, should_be_closer.clone(), vector_size)
                .unwrap();

        assert_eq!(ids.len() - 1, queries.len());

        let (first_lang, first_sql) = queries.get(0).unwrap();
        let (last_lang, last_sql) = queries.get(2).unwrap();

        assert_eq!(first_lang, "language query 3");
        assert_eq!(first_sql, "sql query 3");

        assert_eq!(last_lang, "language query 1");
        assert_eq!(last_sql, "sql query 1");

        // Delete table
        CertifyStore::delete_certified_queries_table(&engine).unwrap();

        // Verify table no longer exists (get queries)
        let result = catch_unwind(AssertUnwindSafe(|| {
            CertifyStore::certify_query(&engine, "test", "test", should_be_closer).unwrap_err();
        }));

        assert!(result.is_err());
    }
}
