use crate::{
    stores::model_store::ModelStore,
    types::{
        errors::SqlgenError,
        formats::judge_output_format_schema,
        structs::{
            engine::TextToSqlEngine,
            instruct_message::{InstructMessage, InstructRole},
            judge_response::JudgeResponse,
        },
        traits::driver::TextInstructDriver,
    },
    utils::sql::get_caught_error_string,
};
use pgrx::{PgTryBuilder, Spi};
use serde_json::{from_str, to_string};
use tera::{Context, Tera};

// Template keys
pub static JUDGE_PROMPT_TEMPLATE_KEY: &str = "judge";
pub static RELEVANT_DDLS_TEMPLATE_KEY: &str = "relevant_ddls";
pub static SIMILAR_QUERIES_TEMPLATE_KEY: &str = "similar_queries";

// Template variable keys
pub static USER_QUERY_VAR_KEY: &str = "user_query";
pub static SQL_QUERY_VAR_KEY: &str = "sql_query";
pub static EXPLAIN_VAR_KEY: &str = "explain";
pub static RELEVANT_DDLS_VAR_KEY: &str = "relevant_ddls";
pub static SIMILAR_QUERIES_VAR_KEY: &str = "similar_queries";

// Template block keys
pub static RELEVANT_DDLS_BLOCK_KEY: &str = "relevant_ddls_block";
pub static SIMILAR_QUERIES_BLOCK_KEY: &str = "similar_queries_block";

pub struct QueryJudgeRunner<'a> {
    engine: &'a TextToSqlEngine,
    model: Box<dyn TextInstructDriver>,
    tera: Tera,
}

impl<'a> QueryJudgeRunner<'a> {
    pub fn new(engine: &'a TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();
        tera.add_raw_template(JUDGE_PROMPT_TEMPLATE_KEY, &engine.judge_query_template)?;
        tera.add_raw_template(RELEVANT_DDLS_TEMPLATE_KEY, &engine.relevant_ddls_template)?;
        tera.add_raw_template(
            SIMILAR_QUERIES_TEMPLATE_KEY,
            &engine.similar_queries_template,
        )?;

        let model = ModelStore::get_text_instruct_model(&engine.instruct_model)?;

        Ok(Self {
            engine,
            tera,
            model,
        })
    }

    fn get_messages(
        &self,
        user_query: &str,
        sql_query: &str,
        explain: &str,
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

        let mut judge_ctx = Context::new();

        judge_ctx.insert(USER_QUERY_VAR_KEY, user_query);
        judge_ctx.insert(SQL_QUERY_VAR_KEY, sql_query);
        judge_ctx.insert(EXPLAIN_VAR_KEY, explain);
        judge_ctx.insert(RELEVANT_DDLS_BLOCK_KEY, &relevant_ddls_block);
        judge_ctx.insert(SIMILAR_QUERIES_BLOCK_KEY, &similar_queries_block);

        let judge_prompt = self.tera.render(JUDGE_PROMPT_TEMPLATE_KEY, &judge_ctx)?;

        Ok(vec![InstructMessage {
            role: InstructRole::User,
            message: judge_prompt,
            output_format: Some(judge_output_format_schema()),
        }])
    }

    pub async fn judge_query(
        &mut self,
        user_query: &str,
        sql_query: &str,
        relevant_ddls: &[String],
        similar_queries: Option<String>,
    ) -> Result<Option<String>, SqlgenError> {
        let explain_query = format!("EXPLAIN (VERBOSE, FORMAT JSON) {}", sql_query);
        let explain_query = match explain_query.ends_with(";") {
            true => explain_query.to_string(),
            false => format!("{};", explain_query),
        };
        let explain = PgTryBuilder::new(|| {
            let result = Spi::explain(&explain_query)?;

            Ok(to_string(&result.0)?)
        })
        .catch_others(|e| Err(SqlgenError::JudgeError(get_caught_error_string(e))))
        .catch_rust_panic(|e| Err(SqlgenError::JudgeError(get_caught_error_string(e))))
        .execute()?;

        let messages = self.get_messages(
            user_query,
            sql_query,
            &explain,
            relevant_ddls,
            similar_queries,
        )?;

        let payload = self.model.get_assistant_response(messages).await?;

        let response = match from_str::<JudgeResponse>(&payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(SqlgenError::JudgeError(e.to_string())),
        }?;

        match response.error.filter(|s| !s.is_empty()) {
            Some(s) => Err(SqlgenError::JudgeError(s)),
            None => match response.counter_example {
                Some(s) => Ok(Some(s)),
                None => Ok(None),
            },
        }
    }
}
