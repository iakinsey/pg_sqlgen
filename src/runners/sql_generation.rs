use crate::types::{
    errors::ModelDriverError,
    structs::{
        engine::TextToSqlEngine,
        instruct_message::{InstructMessage, InstructRole},
    },
    traits::driver::TextInstructDriver,
};
use tera::{Context, Tera};

pub struct SQLGenerationRunner {
    tera: Tera,
    model: Box<dyn TextInstructDriver>,
    engine: TextToSqlEngine,
    system_prompt: String,
}

/*
    Example system template:

    You are a helpful SQL generation system.
    Output in the following format:
    {output_format_description}

    ----------------------------------------

    Example user_template:

    Generate SQL from the following query:
    {user_query}

    {relevant_table_block}

    {similar_queries_block}

    ----------------------------------------

    Example relevant table block:

    Here are some relevant tables:
    {relevant_tables}

    ----------------------------------------

    Example similar queries:

    Here are some similar queries:
    {similar_queries}
*/

pub static OUTPUT_FORMAT_DESCRIPTION: &str = "TODO";

// Template keys
pub static SYSTEM_PROMPT_TEMPLATE_KEY: &str = "system";
pub static USER_PROMPT_TEMPLATE_KEY: &str = "user";
pub static RELEVANT_TABLES_TEMPLATE_KEY: &str = "relevant_tables";
pub static SIMILAR_QUERIES_TEMPLATE_KEY: &str = "similar_queries";

// Template variable keys
pub static OUTPUT_FORMAT_VAR_KEY: &str = "output_format_description";
pub static RELEVANT_TABLES_VAR_KEY: &str = "relevant_tables";
pub static SIMILAR_QUERIES_VAR_KEY: &str = "similar_queries";
pub static USER_QUERY_VAR_KEY: &str = "user_query";

// Template block keys
pub static RELEVANT_TABLES_BLOCK_KEY: &str = "output_format_description";
pub static SIMILAR_QUERIES_BLOCK_KEY: &str = "similar_queries_block";

impl SQLGenerationRunner {
    pub fn new(
        model: Box<dyn TextInstructDriver>,
        engine: TextToSqlEngine,
    ) -> Result<Self, ModelDriverError> {
        let mut tera = Tera::default();

        tera.add_raw_template(SYSTEM_PROMPT_TEMPLATE_KEY, &engine.system_prompt_template)?;
        tera.add_raw_template(USER_PROMPT_TEMPLATE_KEY, &engine.user_prompt_template)?;
        tera.add_raw_template(
            RELEVANT_TABLES_TEMPLATE_KEY,
            &engine.relevant_table_template,
        )?;
        tera.add_raw_template(SIMILAR_QUERIES_TEMPLATE_KEY, &engine.similar_query_template)?;

        let mut sys_ctx = Context::new();
        sys_ctx.insert(OUTPUT_FORMAT_VAR_KEY, OUTPUT_FORMAT_DESCRIPTION);

        let system_prompt = tera.render(SYSTEM_PROMPT_TEMPLATE_KEY, &sys_ctx)?;

        Ok(Self {
            tera: tera,
            model,
            engine,
            system_prompt,
        })
    }

    pub async fn generate_query(
        &mut self,
        user_query: &str,
        relevant_tables: Vec<&str>,
        similar_queries: Vec<&str>,
    ) -> Result<String, ModelDriverError> {
        // Render relevant tables block
        let relevant_tables_block = match relevant_tables.is_empty() {
            true => "".to_string(),
            false => {
                let list = relevant_tables.join("\n");
                let mut ctx = Context::new();
                ctx.insert(RELEVANT_TABLES_VAR_KEY, &list);
                format!(
                    "\n\n{}",
                    self.tera.render(RELEVANT_TABLES_TEMPLATE_KEY, &ctx)?
                )
            }
        };

        // Render similar queries block
        let similar_queries_block = match similar_queries.is_empty() {
            true => "".to_string(),
            false => {
                let list = similar_queries.join("\n");
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
        user_ctx.insert(RELEVANT_TABLES_BLOCK_KEY, &relevant_tables_block);
        user_ctx.insert(SIMILAR_QUERIES_BLOCK_KEY, &similar_queries_block);

        let user_prompt = self.tera.render(USER_PROMPT_TEMPLATE_KEY, &user_ctx)?;
        let messages = vec![
            InstructMessage {
                role: InstructRole::System,
                message: self.system_prompt.clone(),
            },
            InstructMessage {
                role: InstructRole::User,
                message: user_prompt,
            },
        ];

        Ok(self.model.get_assistant_response(messages).await?)
    }
}
