use pgrx::spi::SpiTupleTable;

use crate::{
    types::errors::ModelDriverError,
    utils::sql::{get_column, get_column_optional},
};

pub static DEFAULT_SYSTEM_PROMPT_TEMPLATE: &str = "TODO";
pub static DEFAULT_USER_PROMPT_TEMPLATE: &str = "TODO";
pub static DEFAULT_RELEVANT_TABLES_TEMPLATE: &str = "TODO";
pub static DEFAULT_SIMILAR_QUERIES_TEMPLATE: &str = "TODO";

// TODO update engine store to reflect new changes here
pub struct TextToSqlEngine {
    pub name: String,
    pub schema_name: String,
    pub encoder_model: String,
    pub instruct_model: String,
    pub system_prompt_template: String,
    pub user_prompt_template: String,
    pub relevant_tables_template: String,
    pub similar_queries_template: String,
}

impl TextToSqlEngine {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, ModelDriverError> {
        let system_prompt_template: String =
            match get_column_optional(&row, "system_prompt_template")? {
                Some(v) => v,
                None => DEFAULT_SYSTEM_PROMPT_TEMPLATE.to_string(),
            };
        let user_prompt_template: String = match get_column_optional(&row, "user_prompt_tempate")? {
            Some(v) => v,
            None => DEFAULT_USER_PROMPT_TEMPLATE.to_string(),
        };
        let relevant_tables_template: String =
            match get_column_optional(&row, "relevant_tables_template")? {
                Some(v) => v,
                None => DEFAULT_RELEVANT_TABLES_TEMPLATE.to_string(),
            };
        let similar_queries_template: String =
            match get_column_optional(&row, "similar_queries_template")? {
                Some(v) => v,
                None => DEFAULT_SIMILAR_QUERIES_TEMPLATE.to_string(),
            };

        let name = get_column(&row, "name")?;
        let schema_name = get_column(&row, "schema_name")?;
        let encoder_model = get_column(&row, "encoder_model")?;
        let instruct_model = get_column(&row, "instruct_model")?;

        Ok(Self {
            name,
            schema_name,
            encoder_model,
            instruct_model,
            system_prompt_template,
            user_prompt_template,
            relevant_tables_template,
            similar_queries_template,
        })
    }
}
