use pgrx::prelude::*;

use crate::{
    stores::{
        config_store::{ConfigStore, DEFAULT_ENGINE_CONFIG_KEY},
        engine_store::EngineStore,
    },
    types::{errors::SqlgenError, structs::engine::TextToSqlEngine},
};

// Retrieve engine by name. If one isn't provided, return the default engine.
fn get_engine(name: Option<&str>) -> TextToSqlEngine {
    let engine_name = match name {
        Some(e) => e.to_string(),
        None => match ConfigStore::get_config_value(DEFAULT_ENGINE_CONFIG_KEY) {
            Ok(c) => c,
            Err(SqlgenError::ConfigDoesntExist(_)) => error!("default text to sql engine not set"),
            Err(e) => error!("{}", e),
        },
    };

    EngineStore::get_engine(&engine_name).unwrap_or_else(|e| error!("{}", e))
}

#[pg_schema]
mod sqlgen {
    use crate::{
        api::api::get_engine,
        runners::{
            ddl_filter::DDLFilterRunner, explain::ExplainQueryRunner,
            sql_generation::SQLGenerationRunner, syntax_correction::SyntaxCorrectionRunner,
        },
        stores::{
            config_store::{ConfigStore, DEFAULT_ENGINE_CONFIG_KEY},
            engine_store::EngineStore,
            metadata_store::MetadataStore,
            model_store::ModelStore,
        },
        types::{errors::SqlgenError, structs::engine::TableFilterType},
        utils::{globals::get_runtime, sql::get_current_schema},
    };
    use pgrx::pg_extern;

    // Create a new model instance usable by engines.
    #[pg_extern]
    fn add_model(model_name: &str, config_str: &str) -> Result<(), SqlgenError> {
        ModelStore::create_model_profile(model_name, config_str)?;

        Ok(())
    }

    // Remove model instance.
    #[pg_extern]
    fn remove_model(model_name: &str) -> Result<(), SqlgenError> {
        ModelStore::delete_model_profile(model_name)
    }

    // Call generate() with default engine.
    #[pg_extern(name = "generate")]
    fn generate_1(user_query: &str) -> Result<String, SqlgenError> {
        generate_2(user_query, None)
    }

    // Generate SQL from provided text.
    #[pg_extern(name = "generate")]
    fn generate_2(user_query: &str, engine: Option<&str>) -> Result<String, SqlgenError> {
        let engine = get_engine(engine);
        let mut ddl_filter_runner = DDLFilterRunner::new(&engine)?;
        let mut text_to_sql_runner = SQLGenerationRunner::new(&engine)?;
        let mut syntax_correction_runner = SyntaxCorrectionRunner::new(&engine)?;
        let rt = get_runtime();

        rt.block_on(async {
            let ddls = ddl_filter_runner.generate(user_query).await?;

            let query = text_to_sql_runner
                .generate_query(
                    user_query,
                    ddls.iter().map(|s| s.as_str()).collect(),
                    vec![],
                )
                .await?;

            syntax_correction_runner.correct(query).await
        })
    }

    // Use explain_query() with default engine.
    #[pg_extern(name = "explain_query")]
    fn explain_query_1(sql_query: &str) -> Result<String, SqlgenError> {
        explain_query_2(sql_query, None)
    }

    // Explains how a given an sql query works in natural language.
    #[pg_extern(name = "explain_query")]
    fn explain_query_2(sql_query: &str, engine: Option<&str>) -> Result<String, SqlgenError> {
        let engine = get_engine(engine);
        let mut explain_runner = ExplainQueryRunner::new(engine)?;
        let rt = get_runtime();

        rt.block_on(async { explain_runner.explain(sql_query).await })
    }

    // Sets default engine to be used by functions like generate() and
    // explain_query().
    #[pg_extern]
    fn set_default_engine(engine: &str) -> Result<(), SqlgenError> {
        ConfigStore::set_config_value(DEFAULT_ENGINE_CONFIG_KEY, engine)
    }

    // Removes default engine.
    #[pg_extern]
    fn remove_default_engine() -> Result<(), SqlgenError> {
        ConfigStore::remove_config_value(DEFAULT_ENGINE_CONFIG_KEY)
    }

    // Overloads create_engine() without providing prompt arguments.
    #[pg_extern(name = "create_engine")]
    fn create_engine_3(
        name: &str,
        instruct_model: &str,
        encoder_model: &str,
    ) -> Result<(), SqlgenError> {
        create_engine(
            name,
            instruct_model,
            encoder_model,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    // Overloads create_engine() without providing prompt arguments.
    #[pg_extern(name = "create_engine")]
    fn create_engine_4(
        name: &str,
        instruct_model: &str,
        encoder_model: &str,
        schema_name: Option<&str>,
    ) -> Result<(), SqlgenError> {
        create_engine(
            name,
            instruct_model,
            encoder_model,
            schema_name,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    // Overloads create_engine() without providing prompt arguments.
    #[pg_extern(name = "create_engine")]
    fn create_engine_5(
        name: &str,
        instruct_model: &str,
        encoder_model: &str,
        schema_name: Option<&str>,
        table_filter_type: Option<TableFilterType>,
    ) -> Result<(), SqlgenError> {
        create_engine(
            name,
            instruct_model,
            encoder_model,
            schema_name,
            table_filter_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    // Creates a new text to sql engine.
    #[pg_extern(name = "create_engine")]
    fn create_engine(
        name: &str,
        instruct_model: &str,
        encoder_model: &str,
        schema_name: Option<&str>,
        table_filter_type: Option<TableFilterType>,
        ddl_prompt_limit: Option<i32>,
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
            Some(s) => s,
            None => TableFilterType::Smart,
        };

        let ddl_prompt_limit = match ddl_prompt_limit {
            Some(i) => i,
            None => 128,
        };

        ModelStore::get_model_profile(instruct_model)?;
        let encoder = ModelStore::get_text_encoder_model(encoder_model)?;

        EngineStore::create_engine(
            name,
            &schema_name,
            encoder_model,
            instruct_model,
            table_filter_type.to_str(),
            ddl_prompt_limit,
            system_prompt_template,
            user_prompt_template,
            relevant_ddls_template,
            similar_queries_template,
            filter_ddls_template,
            syntax_correction_template,
            explain_query_template,
        )?;

        let rt = get_runtime();

        rt.block_on(async { MetadataStore::initialize_metadata(name, &schema_name, encoder).await })
    }

    #[pg_extern]
    fn remove_engine(name: &str) -> Result<(), SqlgenError> {
        let engine = EngineStore::get_engine(name)?;

        MetadataStore::remove_metadata(name, &engine.encoder_model)?;
        EngineStore::remove_engine(name)
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::Spi;
    use serde_json::to_string;

    use crate::{pg_test, types::structs::generate_response::GenerateResponse};

    #[pg_test]
    fn test_add_list_and_remove_models() {
        let add_query = "SELECT sqlgen.add_model($1, sqlgen.stub_config('test'))";
        let get_query = "SELECT * FROM sqlgen.models WHERE model_name = $1";
        let remove_query = "SELECT sqlgen.remove_model($1)";
        let model_name = "test_model_name";

        Spi::connect(|client| {
            client
                .select(add_query, None, &[model_name.into()])
                .unwrap();
        });

        let count = Spi::connect(|client| {
            client
                .select(get_query, None, &[model_name.into()])
                .unwrap()
                .count()
        });

        assert_eq!(count, 1);

        Spi::connect(|client| {
            client
                .select(remove_query, None, &[model_name.into()])
                .unwrap();
        });

        let count = Spi::connect(|client| {
            client
                .select(get_query, None, &[model_name.into()])
                .unwrap()
                .count()
        });

        assert_eq!(count, 0);
    }

    #[pg_test]
    fn test_generate() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let user_query = "I am a user query.";
        let expected_query = "SELECT 'Hello world!'";
        let model_response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let generate_query = "SELECT sqlgen.generate($1, $2);";

        Spi::connect(|client| {
            client
                .select(
                    create_model_query,
                    None,
                    &[model_name.into(), model_response_json.into()],
                )
                .unwrap();
        });

        Spi::connect(|client| {
            client
                .select(
                    create_engine_query,
                    None,
                    &[engine_name.into(), model_name.into()],
                )
                .unwrap();
        });

        let generated_query: String = Spi::connect(|client| {
            client
                .select(
                    generate_query,
                    None,
                    &[user_query.into(), engine_name.into()],
                )
                .unwrap()
                .first()
                .get_one::<String>()
                .unwrap()
                .expect("generate returned NULL")
        });

        assert_eq!(expected_query, generated_query);
    }

    #[pg_test]
    fn test_get_default_engine() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let expected_query = "SELECT 'Hello world!'";
        let model_response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let set_default_engine_query = "SELECT sqlgen.set_default_engine($1)";
        let generate_query = "SELECT sqlgen.generate('example query');";

        Spi::connect(|client| {
            client
                .select(
                    create_model_query,
                    None,
                    &[model_name.into(), model_response_json.into()],
                )
                .unwrap();
        });

        Spi::connect(|client| {
            client
                .select(
                    create_engine_query,
                    None,
                    &[engine_name.into(), model_name.into()],
                )
                .unwrap();
        });

        Spi::connect(|client| {
            client
                .select(set_default_engine_query, None, &[engine_name.into()])
                .unwrap();
        });

        let generated_query: String = Spi::connect(|client| {
            client
                .select(generate_query, None, &[])
                .unwrap()
                .first()
                .get_one::<String>()
                .unwrap()
                .expect("generate returned NULL")
        });

        assert_eq!(generated_query, expected_query);
    }

    #[pg_test]
    fn test_create_and_remove_engine() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let expected_query = "SELECT 'Hello world!'";
        let model_response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let get_engine_query = "SELECT * FROM sqlgen.engines WHERE engine_name = $1";
        let remove_engine_query = "SELECT sqlgen.remove_engine($1);";

        Spi::connect(|client| {
            client
                .select(
                    create_model_query,
                    None,
                    &[model_name.into(), model_response_json.into()],
                )
                .unwrap();
        });

        Spi::connect(|client| {
            client
                .select(
                    create_engine_query,
                    None,
                    &[engine_name.into(), model_name.into()],
                )
                .unwrap();
        });

        let count = Spi::connect(|client| {
            client
                .select(get_engine_query, None, &[engine_name.into()])
                .unwrap()
                .count()
        });

        assert_eq!(count, 1);

        Spi::connect(|client| {
            client
                .select(remove_engine_query, None, &[engine_name.into()])
                .unwrap();
        });

        let count = Spi::connect(|client| {
            client
                .select(get_engine_query, None, &[engine_name.into()])
                .unwrap()
                .count()
        });

        assert_eq!(count, 0);
    }
}
