use pgrx::spi::SpiTupleTable;

use crate::{
    stores::model_store::ModelStore,
    types::errors::{ModelDriverError, StoreError},
    utils::sql::{get_column, get_column_optional},
};

pub static DEFAULT_GENERATE_PROMPT: &str = "TODO";
pub static DEFAULT_FILTER_PROMPT: &str = "TODO";

// TODO update engine store to reflect new changes here
pub struct TextToSqlEngine {
    pub name: String,
    pub schema_name: String,
    pub encoder_model: String,
    pub instruct_model: String,
    pub system_prompt_template: String,
    pub user_prompt_template: String,
    pub relevant_table_template: String,
    pub similar_query_template: String,
}

impl TextToSqlEngine {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, ModelDriverError> {
        let name: String = get_column(&row, "name")?;
        let schema_name: String = get_column(&row, "schema_name")?;
        let encoder_model: String = get_column(&row, "encoder_model")?;
        let instruct_model: String = get_column(&row, "instruct_model")?;
        let generate_prompt: String = match get_column_optional(&row, "generate_prompt")? {
            Some(v) => v,
            None => DEFAULT_GENERATE_PROMPT.to_string(),
        };
        let filter_prompt: String = match get_column_optional(&row, "filter_prompt")? {
            Some(v) => v,
            None => DEFAULT_FILTER_PROMPT.to_string(),
        };

        Ok(Self {
            name,
            schema_name,
            encoder_model,
            instruct_model,
            generate_prompt,
            filter_prompt,
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, StoreError> {
        let instruct_model = ModelStore::get_text_instruct_model(&self.instruct_model)?;

        unimplemented!()
    }

    pub async fn execute(&self, prompt: &str) {
        unimplemented!()
    }
}
