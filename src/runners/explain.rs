use crate::{
    stores::model_store::ModelStore,
    types::{
        errors::SqlgenError,
        formats::explain_output_format_schema,
        structs::{
            engine::TextToSqlEngine,
            explain_response::ExplainResponse,
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::TextInstructDriver,
    },
};
use pgrx::Spi;
use serde_json::from_str;
use tera::{Context, Tera};

pub static USER_PROMPT_TEMPLATE_KEY: &str = "explain";
pub static SQL_QUERY_VAR_KEY: &str = "sql_query";
pub static EXPLAIN_VAR_KEY: &str = "explain";

// Outputs an explanation in natural language of how a given SQL query works.
// Both the query and associated EXPLAIN statement are provided to the model.
pub struct ExplainQueryRunner {
    tera: Tera,
    model: Box<dyn TextInstructDriver>,
}

impl ExplainQueryRunner {
    pub fn new(engine: TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();
        let model = ModelStore::get_text_instruct_model(&engine.instruct_model)?;

        tera.add_raw_template(USER_PROMPT_TEMPLATE_KEY, &engine.explain_query_template)?;

        Ok(Self { tera, model })
    }

    // Main entrypoint for runner.
    pub async fn explain(&mut self, sql_query: &str) -> Result<String, SqlgenError> {
        let explain_query = self.get_explain_query(sql_query)?;
        let mut ctx = Context::new();

        ctx.insert(SQL_QUERY_VAR_KEY, &sql_query);
        ctx.insert(EXPLAIN_VAR_KEY, &explain_query);

        let prompt = self.tera.render(USER_PROMPT_TEMPLATE_KEY, &ctx)?;

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: prompt,
            output_format: Some(explain_output_format_schema()),
        }];

        let payload = self.model.get_assistant_response(messages).await?;
        let response = match from_str::<ExplainResponse>(&payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(SqlgenError::GenerateParseError(e.to_string())),
        }?;

        match response.error.filter(|s| !s.is_empty()) {
            Some(s) => Err(SqlgenError::GenerateError(s)),
            None => match response.text {
                Some(s) => Ok(s),
                None => Err(SqlgenError::EmptyResponse),
            },
        }
    }

    fn get_explain_query(&self, sql_query: &str) -> Result<String, SqlgenError> {
        let query = format!("EXPLAIN {};", sql_query);

        Spi::connect(|client| {
            let rows = client.select(&query, None, &[])?;
            let mut values = vec![];

            for row in rows {
                let text = row.get::<String>(1)?;

                if let Some(text) = text {
                    values.push(text)
                }
            }

            Ok(values.join("\n"))
        })
    }
}
