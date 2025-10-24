// TODO

use pgrx::Spi;

use crate::types::{errors::SqlgenError, structs::engine::TextToSqlEngine};

pub struct EngineStore {}

impl EngineStore {
    pub fn create_engine(
        engine_name: &str,
        schema_name: &str,
        encoder_model: &str,
        instruct_model: &str,
        system_prompt_template: Option<&str>,
        user_prompt_template: Option<&str>,
        relevant_tables_template: Option<&str>,
        similar_queries_template: Option<&str>,
        filter_prompt_template: Option<&str>,
        table_filter_type: &str,
    ) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.create_engine($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                engine_name.into(),
                schema_name.into(),
                encoder_model.into(),
                instruct_model.into(),
                system_prompt_template.into(),
                user_prompt_template.into(),
                relevant_tables_template.into(),
                similar_queries_template.into(),
                filter_prompt_template.into(),
                table_filter_type.into(),
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

    pub fn get_engine(engine_name: &str) -> Result<TextToSqlEngine, SqlgenError> {
        let query = "SELECT sqlgen_internal.get_engine($1)";

        let result = Spi::connect(|client| {
            let row = client.select(query, None, &[engine_name.into()])?;

            if row.is_empty() {
                return Err(SqlgenError::ModelDoesntExist(engine_name.to_string()));
            }

            Ok(TextToSqlEngine::from_row(row)?)
        });

        Ok(result?)
    }
}
