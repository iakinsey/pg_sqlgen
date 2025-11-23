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

pub struct SyntaxCorrectionRunner {
    tera: Tera,
    model: Box<dyn TextInstructDriver>,
    engine: TextToSqlEngine,
}

impl SyntaxCorrectionRunner {
    pub fn new(engine: TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();
        let model = ModelStore::get_text_instruct_model(&engine.instruct_model)?;

        tera.add_raw_template(
            SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY,
            &engine.syntax_correction_template,
        )?;

        Ok(Self {
            tera,
            model,
            engine,
        })
    }

    pub async fn correct(&mut self, query: String) -> Result<String, SqlgenError> {
        let id = get_unique_prepared_statement_id();
        let query = format!("PREPARE {} AS {}", id, query);
        let query = match query.ends_with(";") {
            true => query,
            false => format!("{};", query),
        };

        let error = match Spi::run(&query) {
            Ok(_) => {
                let dealloc_query = format!("DEALLOCATE {}", id);
                Spi::run(&dealloc_query)?;

                return Ok(query);
            }
            Err(e) => e.to_string(),
        };

        let mut prompt_ctx = Context::new();

        prompt_ctx.insert(ERROR_MESSAGE_VAR_KEY, &error);

        let prompt = self
            .tera
            .render(SYNTAX_CORRECTION_PROMPT_TEMPLATE_KEY, &prompt_ctx)?;

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: prompt,
        }];

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
