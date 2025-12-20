use crate::{
    stores::{
        config_store::{ConfigStore, DEFAULT_ENGINE_CONFIG_KEY},
        engine_store::EngineStore,
    },
    types::{errors::SqlgenError, structs::engine::TextToSqlEngine},
};
use pgrx::prelude::*;

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
        api::public::get_engine,
        runners::{
            ddl_filter::DDLFilterRunner, explain::ExplainQueryRunner, judge::QueryJudgeRunner,
            sql_generation::SQLGenerationRunner, syntax_correction::SyntaxCorrectionRunner,
        },
        stores::{
            certify_store::CertifyStore,
            config_store::{ConfigStore, DEFAULT_ENGINE_CONFIG_KEY},
            engine_store::EngineStore,
            metadata_store::MetadataStore,
            model_store::ModelStore,
        },
        types::errors::SqlgenError,
        utils::{globals::get_runtime, sql::get_prepare_error},
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
        let model = ModelStore::get_text_encoder_model(&engine.instruct_model)?;
        let rt = get_runtime();

        rt.block_on(async {
            let ddls = ddl_filter_runner.generate(user_query).await?;
            let query_vector = model.encode(user_query).await?;

            // Diminishing returns on query accuracy when similiar queries > 3.
            // TODO provide a config option
            let similar_queries =
                CertifyStore::get_formatted_certified_queries(&engine.name, query_vector, 3)?;
            let mut query = text_to_sql_runner
                .generate_query(user_query, &ddls, similar_queries.as_deref())
                .await?;

            query = syntax_correction_runner.correct(query, &ddls).await?;

            if engine.enable_judge {
                let mut judge_runner = QueryJudgeRunner::new(&engine)?;

                let counter_example = judge_runner
                    .judge_query(user_query, &query, &ddls, similar_queries.as_deref())
                    .await?;

                if let Some(counter_example) = counter_example {
                    query = syntax_correction_runner
                        .correct(counter_example, &ddls)
                        .await?;
                }
            }

            Ok(query)
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

    #[pg_extern]
    fn remove_engine(name: &str) -> Result<(), SqlgenError> {
        let engine = EngineStore::get_engine(name)?;

        MetadataStore::remove_metadata(name)?;
        EngineStore::remove_engine(name)?;
        CertifyStore::delete_certified_queries_table(&engine.name)
    }

    // Certifies a functional query so that it can be provided to prompts to boost accuracy.
    #[pg_extern(name = "certify_query")]
    fn certify_query(
        language_query: &str,
        sql_query: &str,
        engine: Option<&str>,
    ) -> Result<String, SqlgenError> {
        let engine = get_engine(engine);
        let rt = get_runtime();

        if let Some(e) = get_prepare_error(sql_query.to_string())? {
            return Err(SqlgenError::Any(e));
        }

        let model = ModelStore::get_text_encoder_model(&engine.encoder_model)?;
        let language_vector = rt.block_on(async { model.encode(language_query).await })?;

        CertifyStore::certify_query(&engine.name, language_query, sql_query, language_vector)
    }

    #[pg_extern(name = "certify_query")]
    fn certify_query_2(language_query: &str, sql_query: &str) -> Result<String, SqlgenError> {
        certify_query(language_query, sql_query, None)
    }

    // Removes certified query by id
    #[pg_extern(name = "decertify_query")]
    fn decertify_query(id: &str, engine: Option<&str>) -> Result<(), SqlgenError> {
        let engine = get_engine(engine);

        CertifyStore::decertify_query(&engine.name, id)
    }

    #[pg_extern(name = "decertify_query")]
    fn decertify_query_1(id: &str) -> Result<(), SqlgenError> {
        decertify_query(id, None)
    }

    #[pg_extern]
    fn add_schema_to_engine(engine: &str, schema: &str) -> Result<(), SqlgenError> {
        let encoder_model_name = EngineStore::get_engine(engine)?.encoder_model;
        let encoder = ModelStore::get_text_encoder_model(&encoder_model_name)?;
        let rt = get_runtime();

        rt.block_on(async {
            EngineStore::add_schema_to_engine(engine, schema)?;
            MetadataStore::add_schema_to_engine(engine, schema, encoder).await
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{PgTryBuilder, Spi};
    use serde_json::to_string;

    use crate::{
        pg_test, stores::engine_store::EngineStore,
        types::structs::generate_response::GenerateResponse, utils::sql::get_caught_error_string,
    };

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

    #[pg_test]
    fn test_certify_decertify() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let sql_query = "SELECT 'Hello world!'";
        let language_query = "hello world query";

        let model_response = GenerateResponse {
            query: Some(sql_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let certify_query = "SELECT sqlgen.certify_query($1, $2, $3);";
        let decertify_query = "SELECT sqlgen.decertify_query($1, $2);";

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

        let query_id: String = Spi::connect(|client| {
            client
                .select(
                    certify_query,
                    None,
                    &[language_query.into(), sql_query.into(), engine_name.into()],
                )
                .unwrap()
                .first()
                .get_one::<String>()
                .unwrap()
                .expect("generate returned NULL")
        });

        Spi::connect(|client| {
            client
                .select(
                    decertify_query,
                    None,
                    &[query_id.clone().into(), engine_name.into()],
                )
                .unwrap();
        });
    }

    #[pg_test]
    fn test_certify_prepare_fail() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let sql_query = "SELECT 123aaaa";
        let language_query = "hello world query";

        let model_response = GenerateResponse {
            query: Some(sql_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let certify_query = "SELECT sqlgen.certify_query($1, $2, $3);";

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

        let error: Option<String> = PgTryBuilder::new(|| {
            Spi::connect(|client| {
                client
                    .select(
                        certify_query,
                        None,
                        &[language_query.into(), sql_query.into(), engine_name.into()],
                    )
                    .unwrap()
                    .first()
                    .get_one::<String>()
                    .unwrap()
                    .expect("generate returned NULL")
            });

            None
        })
        .catch_others(|e| Some(get_caught_error_string(e)))
        .catch_rust_panic(|e| Some(get_caught_error_string(e)))
        .execute();

        assert_eq!(
            error.unwrap(),
            r#"ERRCODE_DATA_EXCEPTION: ERRCODE_SYNTAX_ERROR: trailing junk after numeric literal at or near "123aaaa""#
        );
    }

    #[pg_test]
    fn test_add_schema_to_engine() {
        let engine_name = "engine_name";
        let model_name = "stub_model";
        let expected_query = "SELECT 'Hello world!'";
        let model_response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let model_response_json = to_string(&model_response);
        let secondary_schema_name = "secondary_schema";
        let create_model_query =
            "SELECT sqlgen.add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT sqlgen.create_engine($1, $2, $2)";
        let create_schema_query = format!("CREATE SCHEMA {}", secondary_schema_name);
        let add_schema_query = "SELECT sqlgen.add_schema_to_engine($1, $2)";

        Spi::run(&create_schema_query).unwrap();

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

        Spi::run_with_args(
            add_schema_query,
            &[engine_name.into(), secondary_schema_name.into()],
        )
        .unwrap();

        let engine = EngineStore::get_engine(engine_name).unwrap();

        assert_eq!(engine.schema_names.len(), 2);
    }
}
