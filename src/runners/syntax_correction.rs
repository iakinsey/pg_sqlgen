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
    utils::sql::{get_caught_error_string, get_unique_prepared_statement_id},
};
use pgrx::{PgTryBuilder, Spi};
use serde_json::from_str;
use tera::{Context, Tera};

// Template key
pub static SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY: &str = "error_correction";
pub static RELEVANT_DDLS_TEMPLATE_KEY: &str = "relevant_ddls";

// Template variable keys
pub static ERROR_MESSAGE_VAR_KEY: &str = "error_message";
pub static RELEVANT_DDLS_VAR_KEY: &str = "relevant_ddls";
pub static QUERY_VAR_KEY: &str = "query";

// Template block key
pub static RELEVANT_DDLS_BLOCK_KEY: &str = "relevant_ddls_block";

// Validates whether or not a given SQL query is both syntactically correct and
// semantically resolvable. If not, it pipes the query and associated error into
// the model for correction. Returns corrected query.
//
// TODO allow it to pass through rounds of validation in the event that the
// first pass fails.
pub struct SyntaxCorrectionRunner {
    tera: Tera,
    model: Box<dyn TextInstructDriver>,
    system_prompt: String,
}

impl SyntaxCorrectionRunner {
    pub fn new(engine: &TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();
        let model = ModelStore::get_text_instruct_model(&engine.instruct_model)?;

        tera.add_raw_template(
            SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY,
            &engine.syntax_correction_template,
        )?;
        tera.add_raw_template(RELEVANT_DDLS_TEMPLATE_KEY, &engine.relevant_ddls_template)?;

        let system_prompt = engine.get_generate_system_prompt()?;

        Ok(Self {
            tera,
            model,
            system_prompt,
        })
    }

    fn get_messages(
        &self,
        query: &str,
        error: &str,
        ddls: &[String],
    ) -> Result<Vec<InstructMessage>, SqlgenError> {
        let mut prompt_ctx = Context::new();

        let relevant_ddls_block = match ddls.is_empty() {
            true => "".to_string(),
            false => {
                let list = ddls.join("\n");
                let mut ctx = Context::new();
                ctx.insert(RELEVANT_DDLS_VAR_KEY, &list);
                format!(
                    "\n\n{}",
                    self.tera.render(RELEVANT_DDLS_TEMPLATE_KEY, &ctx)?
                )
            }
        };

        prompt_ctx.insert(RELEVANT_DDLS_BLOCK_KEY, &relevant_ddls_block);
        prompt_ctx.insert(ERROR_MESSAGE_VAR_KEY, error);
        prompt_ctx.insert(QUERY_VAR_KEY, query);

        let prompt = self
            .tera
            .render(SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY, &prompt_ctx)?;

        Ok(vec![
            InstructMessage {
                role: InstructRole::System,
                message: self.system_prompt.clone(),
                output_format: Some(generate_output_format_schema()),
            },
            InstructMessage {
                role: InstructRole::User,
                message: prompt,
                output_format: None,
            },
        ])
    }

    // Main entrypoint for runner.
    pub async fn correct(&mut self, query: String, ddls: &[String]) -> Result<String, SqlgenError> {
        let id = get_unique_prepared_statement_id();
        let prepare_query = format!("PREPARE {} AS {}", id, query);
        let prepare_query = match prepare_query.ends_with(";") {
            true => prepare_query.to_string(),
            false => format!("{};", prepare_query),
        };

        let error: Option<String> = PgTryBuilder::new(|| {
            Spi::run(&prepare_query).unwrap();
            None
        })
        .catch_others(|e| Some(get_caught_error_string(e)))
        .catch_rust_panic(|e| Some(get_caught_error_string(e)))
        .execute();

        let error = match error {
            Some(s) => s,
            None => return Ok(query),
        };

        let messages = self.get_messages(&query, &error, ddls)?;
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
        runners::syntax_correction::SyntaxCorrectionRunner,
        types::structs::generate_response::GenerateResponse,
        utils::{globals::get_runtime, test_utils::create_engine},
    };

    #[pg_test]
    fn test_syntax_prompt_output() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let query = "SELECT 12s;";
        let error = r#"trailing junk after numeric literal at or near "12s""#;
        let response = GenerateResponse {
            query: Some(query.to_string()),
            error: None,
        };
        let ddls = vec!["ddl1".to_string(), "ddl2".to_string(), "ddl3".to_string()];
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let runner = SyntaxCorrectionRunner::new(&engine).unwrap();
        let messages = runner.get_messages(query, error, &ddls).unwrap();
        let user_prompt = messages[1].message.clone();

        assert!(user_prompt.contains(query));
        assert!(user_prompt.contains(error));

        for ddl in &ddls {
            assert!(user_prompt.contains(ddl));
        }
    }

    #[pg_test]
    fn test_syntax_prompt_output_error() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let query = "SELECT 12s;";
        let response = GenerateResponse {
            query: Some(query.to_string()),
            error: None,
        };
        let ddls = vec!["ddl1".to_string(), "ddl2".to_string(), "ddl3".to_string()];
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let mut runner = SyntaxCorrectionRunner::new(&engine).unwrap();
        let rt = get_runtime();

        let result = rt.block_on(async { runner.correct(query.to_string(), &ddls).await.unwrap() });

        assert_eq!(result, query);
    }
}
