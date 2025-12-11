use crate::{
    stores::model_store::ModelStore,
    types::{
        errors::SqlgenError,
        formats::generate_output_format_schema,
        structs::{
            engine::TextToSqlEngine,
            generate_response::GenerateResponse,
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::TextInstructDriver,
    },
};
use serde_json::from_str;
use tera::{Context, Tera};

// Template keys
pub static USER_PROMPT_TEMPLATE_KEY: &str = "user";
pub static RELEVANT_DDLS_TEMPLATE_KEY: &str = "relevant_ddls";
pub static SIMILAR_QUERIES_TEMPLATE_KEY: &str = "similar_queries";

// Template variable keys
pub static RELEVANT_DDLS_VAR_KEY: &str = "relevant_ddls";
pub static SIMILAR_QUERIES_VAR_KEY: &str = "similar_queries";
pub static USER_QUERY_VAR_KEY: &str = "user_query";

// Template block keys
pub static RELEVANT_DDLS_BLOCK_KEY: &str = "relevant_ddls_block";
pub static SIMILAR_QUERIES_BLOCK_KEY: &str = "similar_queries_block";

// Generates an SQL query based on a natural language statement from the user.
// Requires a list of relevant DDLs, which can be retrieved from
// `DDLFilterRunner`.  While parameters ask for similar queries, it is not
// currently implemented.
pub struct SQLGenerationRunner {
    tera: Tera,
    model: Box<dyn TextInstructDriver>,
    system_prompt: String,
}

impl SQLGenerationRunner {
    pub fn new(engine: &TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();

        tera.add_raw_template(USER_PROMPT_TEMPLATE_KEY, &engine.user_prompt_template)?;
        tera.add_raw_template(RELEVANT_DDLS_TEMPLATE_KEY, &engine.relevant_ddls_template)?;
        tera.add_raw_template(
            SIMILAR_QUERIES_TEMPLATE_KEY,
            &engine.similar_queries_template,
        )?;

        let model = ModelStore::get_text_instruct_model(&engine.instruct_model)?;
        let system_prompt = engine.get_generate_system_prompt()?;

        Ok(Self {
            tera,
            model,
            system_prompt,
        })
    }

    fn get_messages(
        &self,
        user_query: &str,
        relevant_ddls: &[String],
        similar_queries: Option<String>,
    ) -> Result<Vec<InstructMessage>, SqlgenError> {
        // Render relevant tables block
        let relevant_ddls_block = match relevant_ddls.is_empty() {
            true => "".to_string(),
            false => {
                let list = relevant_ddls.join("\n");
                let mut ctx = Context::new();
                ctx.insert(RELEVANT_DDLS_VAR_KEY, &list);
                format!(
                    "\n\n{}",
                    self.tera.render(RELEVANT_DDLS_TEMPLATE_KEY, &ctx)?
                )
            }
        };

        // Render similar queries block
        let similar_queries_block = match similar_queries {
            None => "".to_string(),
            Some(list) => {
                let mut ctx = Context::new();
                ctx.insert(SIMILAR_QUERIES_VAR_KEY, &list);

                format!(
                    "\n\n{}",
                    self.tera.render(SIMILAR_QUERIES_TEMPLATE_KEY, &ctx)?
                )
            }
        };

        // Render user prompt
        let mut user_ctx = Context::new();

        user_ctx.insert(USER_QUERY_VAR_KEY, user_query);
        user_ctx.insert(RELEVANT_DDLS_BLOCK_KEY, &relevant_ddls_block);
        user_ctx.insert(SIMILAR_QUERIES_BLOCK_KEY, &similar_queries_block);

        let user_prompt = self.tera.render(USER_PROMPT_TEMPLATE_KEY, &user_ctx)?;

        Ok(vec![
            InstructMessage {
                role: InstructRole::System,
                message: self.system_prompt.clone(),
                output_format: Some(generate_output_format_schema()),
            },
            InstructMessage {
                role: InstructRole::User,
                message: user_prompt,
                output_format: None,
            },
        ])
    }

    // Main entrypoint for runner.
    pub async fn generate_query(
        &mut self,
        user_query: &str,
        relevant_ddls: &[String],
        similar_queries: Option<String>,
    ) -> Result<String, SqlgenError> {
        let messages = self.get_messages(user_query, relevant_ddls, similar_queries)?;
        let payload = self.model.get_assistant_response(messages).await?;
        let response = match from_str::<GenerateResponse>(&payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(SqlgenError::GenerateParseError(e.to_string())),
        }?;

        match response.error.filter(|s| !s.is_empty()) {
            Some(s) => Err(SqlgenError::GenerateError(s)),
            None => match response.query {
                Some(s) => Ok(s),
                None => Err(SqlgenError::EmptyResponse),
            },
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use serde_json::to_string;

    use crate::{
        pg_test,
        runners::sql_generation::SQLGenerationRunner,
        stores::{metadata_store::MetadataStore, model_store::ModelStore},
        types::structs::generate_response::GenerateResponse,
        utils::{
            globals::get_runtime,
            test_utils::{create_engine, create_schema},
        },
    };

    #[pg_test]
    fn test_generation_prompt_output() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_query = "SELECT * from test;";
        let response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let user_query = "Top 10 Maxwell the cat memes.";
        let relevant_ddls: Vec<String> = vec!["ddl1", "ddl2", "ddl3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let similar_queries = "similar queries";
        let ddls_part = relevant_ddls.join("\n");

        create_schema(schema_name);

        let engine = SQLGenerationRunner::new(&engine).unwrap();
        let messages = engine
            .get_messages(
                user_query,
                &relevant_ddls,
                Some(similar_queries.to_string()),
            )
            .unwrap();
        let user_prompt = messages[1].message.clone();

        assert!(user_prompt.contains(user_query));
        assert!(user_prompt.contains(&ddls_part));
        assert!(user_prompt.contains(similar_queries));
    }

    #[pg_test]
    fn test_generate_query_success() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_query = "SELECT * from test;";
        let response = GenerateResponse {
            query: Some(expected_query.to_string()),
            error: None,
        };
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();

        create_schema(schema_name);

        let generate_response = rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, &[schema_name], encoder)
                .await
                .unwrap();
            let mut engine = SQLGenerationRunner::new(&engine).unwrap();
            engine
                .generate_query("test_query", &vec!["".to_string()], None)
                .await
                .unwrap()
        });

        assert_eq!(expected_query, generate_response);
    }

    #[pg_test]
    fn test_generate_query_model_outputs_error() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_error_text = "example error";
        let response = GenerateResponse {
            query: None,
            error: Some(expected_error_text.to_string()),
        };
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();

        create_schema(schema_name);

        let generate_error = rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, &[schema_name], encoder)
                .await
                .unwrap();
            let mut engine = SQLGenerationRunner::new(&engine).unwrap();
            engine
                .generate_query("test_query", &vec!["".to_string()], Some("".to_string()))
                .await
                .unwrap_err()
        });

        assert_eq!(expected_error_text, generate_error.to_string());
    }

    #[pg_test]
    fn test_generate_query_malformed_response() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let engine = create_engine(engine_name, schema_name, "smart", "}{");
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();

        create_schema(schema_name);

        let generate_error = rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, &[schema_name], encoder)
                .await
                .unwrap();
            let mut engine = SQLGenerationRunner::new(&engine).unwrap();
            engine
                .generate_query("test_query", &vec!["".to_string()], Some("".to_string()))
                .await
                .unwrap_err()
        });

        assert!(generate_error
            .to_string()
            .starts_with("failed to parse model output when generating output: "));
    }

    #[pg_test]
    fn test_generate_query_empty_response() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let response = GenerateResponse {
            query: None,
            error: None,
        };
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();

        create_schema(schema_name);

        let generate_response = rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, &[schema_name], encoder)
                .await
                .unwrap();
            let mut engine = SQLGenerationRunner::new(&engine).unwrap();
            engine
                .generate_query("test_query", &vec!["".to_string()], Some("".to_string()))
                .await
                .unwrap_err()
        });

        assert_eq!(
            "SQL generation model returned no response",
            generate_response.to_string()
        );
    }
}
