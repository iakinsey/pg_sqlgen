use pgrx::prelude::*;
use tokio::runtime::Runtime;

use crate::{
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
    types::{
        errors::SqlgenError,
        structs::engine::{TableFilterType, TextToSqlEngine},
    },
    utils::sql::get_current_schema,
};

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

#[pg_extern]
fn add_model(model_name: &str, config_str: &str) {
    ModelStore::create_model_profile(model_name, config_str).unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern]
fn remove_model(model_name: &str) {
    ModelStore::delete_model_profile(model_name).unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern(name = "generate")]
fn generate_1(user_query: &str) -> String {
    generate_2(user_query, None)
}

#[pg_extern(name = "generate")]
fn generate_2(user_query: &str, engine: Option<&str>) -> String {
    let engine = get_engine(engine);
    let mut ddl_filter_runner =
        DDLFilterRunner::new(engine.clone()).unwrap_or_else(|e| error!("{}", e));
    let mut text_to_sql_runner =
        SQLGenerationRunner::new(engine.clone()).unwrap_or_else(|e| error!("{}", e));
    let mut syntax_correction_runner =
        SyntaxCorrectionRunner::new(engine).unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async {
        let ddls = ddl_filter_runner
            .generate(user_query)
            .await
            .unwrap_or_else(|e| error!("{}", e));

        let query = text_to_sql_runner
            .generate_query(
                user_query,
                ddls.iter().map(|s| s.as_str()).collect(),
                vec![],
            )
            .await
            .unwrap_or_else(|e| error!("{}", e));

        syntax_correction_runner
            .correct(&query)
            .await
            .unwrap_or_else(|e| error!("{}", e))
    })
}

#[pg_extern(name = "explain_query")]
fn explain_query_1(sql_query: &str) -> String {
    explain_query_2(sql_query, None)
}

#[pg_extern(name = "explain_query")]
fn explain_query_2(sql_query: &str, engine: Option<&str>) -> String {
    let engine = get_engine(engine);
    let mut explain_runner = ExplainQueryRunner::new(engine).unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async {
        explain_runner
            .explain(sql_query)
            .await
            .unwrap_or_else(|e| error!("{}", e))
    })
}

#[pg_extern]
fn set_default_engine(engine: &str) {
    ConfigStore::set_config_value(DEFAULT_ENGINE_CONFIG_KEY, engine)
        .unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern]
fn remove_default_engine() {
    ConfigStore::remove_config_value(DEFAULT_ENGINE_CONFIG_KEY).unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern(name = "create_engine")]
fn create_engine_3(name: &str, instruct_model: &str, encoder_model: &str) {
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
) {
    let schema_name = match schema_name {
        Some(s) => s.to_string(),
        None => get_current_schema().unwrap_or_else(|e| error!("{}", e)),
    };

    let table_filter_type = match table_filter_type {
        Some(s) => s,
        None => TableFilterType::Smart,
    };

    let ddl_prompt_limit = match ddl_prompt_limit {
        Some(i) => i,
        None => 128,
    };

    ModelStore::get_model_profile(instruct_model).unwrap_or_else(|e| error!("{}", e));
    let encoder =
        ModelStore::get_text_encoder_model(encoder_model).unwrap_or_else(|e| error!("{}", e));

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
    )
    .unwrap_or_else(|e| error!("{}", e));

    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async {
        MetadataStore::initialize_metadata(name, &schema_name, encoder)
            .await
            .unwrap_or_else(|e| error!("{}", e));
    })
}

#[pg_extern]
fn remove_engine(name: &str) {
    let engine = EngineStore::get_engine(name).unwrap_or_else(|e| error!("{}", e));

    MetadataStore::remove_metadata(name, &engine.encoder_model).unwrap_or_else(|e| error!("{}", e));
    EngineStore::remove_engine(name).unwrap_or_else(|e| error!("{}", e));
}

// TODO test each function
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::Spi;
    use serde_json::to_string;

    use crate::{pg_test, types::structs::generate_response::GenerateResponse};

    #[pg_test]
    fn test_add_list_and_remove_models() {
        let add_query = "SELECT add_model($1, sqlgen.stub_config('test')";
        let get_query = "SELECT * FROM sqlgen.models WHERE model_name = $1";
        let remove_query = "SELECT remove_model($1)";
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
    fn test_generate_and_execute() {
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
            "SELECT add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT create_engine($1, $2, $2)";
        let generate_query = "SELECT generate($1, $2);";

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
            "SELECT add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT create_engine($1, $2, $2)";
        let set_default_engine_query = "SELECT set_default_engine($1)";
        let generate_query = "SELECT generate('example query');";

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
            "SELECT add_model($1, sqlgen.stub_config($2, ARRAY[0.0, 0.5, 1.0]::REAL[]))";
        let create_engine_query = "SELECT create_engine($1, $2, $2)";
        let get_engine_query = "SELECT * FROM sqlgen.engines WHERE engine_name = $1";
        let remove_engine_query = "SELECT remove_engine($1);";

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
