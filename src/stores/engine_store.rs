use pgrx::Spi;

use crate::{
    types::{errors::SqlgenError, structs::engine::TextToSqlEngine},
    utils::sql::get_column_heap,
};

// Handles state management for engines.
pub struct EngineStore {}

#[allow(clippy::too_many_arguments)]
impl EngineStore {
    pub fn create_engine(
        engine_name: &str,
        schema_names: &[&str],
        encoder_model: &str,
        instruct_model: &str,
        table_filter_type: &str,
        column_filter_limit: i32,
        error_correction_rounds: i32,
        system_prompt_template: Option<&str>,
        user_prompt_template: Option<&str>,
        relevant_ddls_template: Option<&str>,
        similar_queries_template: Option<&str>,
        filter_ddls_template: Option<&str>,
        syntax_correction_template: Option<&str>,
        explain_query_template: Option<&str>,
        judge_query_template: Option<&str>,
    ) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.create_engine($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            &[
                engine_name.into(),
                schema_names.into(),
                encoder_model.into(),
                instruct_model.into(),
                table_filter_type.into(),
                column_filter_limit.into(),
                error_correction_rounds.into(),
                system_prompt_template.into(),
                user_prompt_template.into(),
                relevant_ddls_template.into(),
                similar_queries_template.into(),
                filter_ddls_template.into(),
                syntax_correction_template.into(),
                explain_query_template.into(),
                judge_query_template.into(),
            ],
        )?;

        Ok(())
    }

    pub fn remove_engine(engine_name: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.remove_engine($1)",
            &[engine_name.into()],
        )?;

        Ok(())
    }

    pub fn add_schema_to_engine(engine_name: &str, schema_name: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.add_schema_to_engine($1, $2);",
            &[engine_name.into(), schema_name.into()],
        )?;

        Ok(())
    }

    pub fn get_engine(engine_name: &str) -> Result<TextToSqlEngine, SqlgenError> {
        let query = "SELECT (sqlgen_internal.get_engine($1)).*";

        let result = Spi::connect(|client| {
            let row = client.select(query, None, &[engine_name.into()])?;

            if row.is_empty() {
                return Err(SqlgenError::EngineDoesntExist(engine_name.to_string()));
            }

            TextToSqlEngine::from_row(row.first())
        });

        result
    }
    pub fn list_engines() -> Result<Vec<String>, SqlgenError> {
        let query = "SELECT engine_name FROM sqlgen_internal.list_engines();";

        Spi::connect(|client| {
            let rows = client.select(query, None, &[])?;
            let mut results = Vec::new();

            for row in rows {
                let engine_name: String = get_column_heap(&row, "engine_name")?;

                results.push(engine_name)
            }

            Ok(results)
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {

    use pgrx::{PgTryBuilder, Spi};
    use serde_json::to_string;

    use crate::{
        pg_test,
        stores::engine_store::EngineStore,
        types::{
            errors::SqlgenError,
            structs::{model_profile::ModelConfig, profiles::StubConfig},
        },
    };

    #[pg_test]
    fn test_engine_crud() {
        let name = "test_engine_name";
        let schema_name = "test_schema_name";
        let secondary_schema_name = "secondary_schema";
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

        Spi::run(format!("CREATE SCHEMA {};", schema_name).as_str()).unwrap();
        Spi::run(format!("CREATE SCHEMA {};", secondary_schema_name).as_str()).unwrap();

        Spi::run_with_args(
            "SELECT sqlgen_internal.create_model($1, $2::JSONB);",
            &[encoder_model_name.into(), encoder_profile_json.into()],
        )
        .unwrap();

        Spi::run_with_args(
            "SELECT sqlgen_internal.create_model($1, $2::JSONB);",
            &[instruct_model_name.into(), instruct_profile_json.into()],
        )
        .unwrap();

        EngineStore::create_engine(
            name,
            &[schema_name, secondary_schema_name],
            encoder_model_name,
            instruct_model_name,
            table_filter_type,
            3,
            128,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let engine = EngineStore::get_engine(name).unwrap();

        assert_eq!(name, engine.name);
        assert_eq!(
            vec![schema_name, secondary_schema_name],
            engine.schema_names
        );
        assert_eq!(encoder_model_name, engine.encoder_model);
        assert_eq!(instruct_model_name, engine.instruct_model);
        assert_eq!(table_filter_type, engine.filter_type.to_str());

        EngineStore::remove_engine(name).unwrap();

        match EngineStore::get_engine(name) {
            Ok(_) => panic!("engine still exists after calling remove_engine"),
            Err(SqlgenError::EngineDoesntExist(_)) => assert!(true),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[pg_test]
    #[should_panic(expected = "Schemas do not exist: {secondary_schema,test_schema_name}")]
    fn test_create_engine_no_schema() {
        let name = "test_engine_name";
        let schema_name = "test_schema_name";
        let secondary_schema_name = "secondary_schema";
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
            "SELECT sqlgen_internal.create_model($1, $2::JSONB);",
            &[encoder_model_name.into(), encoder_profile_json.into()],
        )
        .unwrap();

        Spi::run_with_args(
            "SELECT sqlgen_internal.create_model($1, $2::JSONB);",
            &[instruct_model_name.into(), instruct_profile_json.into()],
        )
        .unwrap();

        EngineStore::create_engine(
            name,
            &[schema_name, secondary_schema_name],
            encoder_model_name,
            instruct_model_name,
            table_filter_type,
            3,
            128,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    }

    #[pg_test]
    fn test_create_engine_model_doesnt_exist() {
        let name = "test_engine_name";
        let schema_name = "test_schema_name";
        let table_filter_type = "smart";
        let encoder_model_name = "test_encoder_model_name";
        let instruct_model_name = "test_instruct_model_name";
        let result = PgTryBuilder::new(|| {
            EngineStore::create_engine(
                name,
                &[schema_name],
                encoder_model_name,
                instruct_model_name,
                table_filter_type,
                3,
                128,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        })
        .catch_others(|_| Err(SqlgenError::Any("test".to_string())))
        .execute();

        assert!(result.is_err())
    }
}
