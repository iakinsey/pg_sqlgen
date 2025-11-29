use crate::{
    stores::model_store::ModelStore,
    types::{
        errors::SqlgenError,
        structs::{
            engine::TextToSqlEngine,
            generate_response::GenerateResponse,
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::TextInstructDriver,
    },
    utils::sql::get_unique_prepared_statement_id,
};
use pgrx::Spi;
use serde_json::from_str;
use tera::{Context, Tera};

// Template key
pub static SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY: &str = "error_correction";

// Template variable keys
pub static ERROR_MESSAGE_VAR_KEY: &str = "error_message";
pub static QUERY_VAR_KEY: &str = "query";

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
        let system_prompt = engine.get_generate_system_prompt()?;

        Ok(Self {
            tera,
            model,
            system_prompt,
        })
    }

    fn get_messages(&self, query: &str, error: &str) -> Result<Vec<InstructMessage>, SqlgenError> {
        let mut prompt_ctx = Context::new();

        prompt_ctx.insert(ERROR_MESSAGE_VAR_KEY, error);
        prompt_ctx.insert(QUERY_VAR_KEY, query);

        let prompt = self
            .tera
            .render(SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY, &prompt_ctx)?;

        Ok(vec![
            InstructMessage {
                role: InstructRole::System,
                message: self.system_prompt.clone(),
            },
            InstructMessage {
                role: InstructRole::User,
                message: prompt,
            },
        ])
    }

    // Main entrypoint for runner.
    pub async fn correct(&mut self, query: String) -> Result<String, SqlgenError> {
        let id = get_unique_prepared_statement_id();
        let prepare_query = format!("PREPARE {} AS {}", id, query);
        let prepare_query = match prepare_query.ends_with(";") {
            true => prepare_query.to_string(),
            false => format!("{};", prepare_query),
        };

        let error = match Spi::run(&prepare_query) {
            Ok(_) => {
                let dealloc_query = format!("DEALLOCATE {}", id);
                Spi::run(&dealloc_query)?;

                return Ok(query);
            }
            Err(e) => e.to_string(),
        };

        let messages = self.get_messages(&query, &error)?;
        let payload = self.model.get_assistant_response(messages).await?;
        let response = match from_str::<GenerateResponse>(&payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(SqlgenError::GenerateParseError(e.to_string())),
        }?;

        match response.error {
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
        pg_test, runners::syntax_correction::SyntaxCorrectionRunner,
        types::structs::engine::OUTPUT_FORMAT_DESCRIPTION,
        types::structs::generate_response::GenerateResponse, utils::test_utils::create_engine,
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
        let response_str = to_string(&response).unwrap();
        let engine = create_engine(engine_name, schema_name, "smart", &response_str);
        let runner = SyntaxCorrectionRunner::new(&engine).unwrap();

        let messages = runner.get_messages(query, error).unwrap();
        let system_prompt = messages[0].message.clone();
        let user_prompt = messages[1].message.clone();

        assert!(system_prompt.contains(OUTPUT_FORMAT_DESCRIPTION));
        assert!(user_prompt.contains(query));
        assert!(user_prompt.contains(error));
    }
}
