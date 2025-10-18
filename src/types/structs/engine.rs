use pgrx::{spi::SpiTupleTable, PostgresEnum};

use crate::{
    types::errors::ModelDriverError,
    utils::sql::{get_column, get_column_optional},
};

pub static DEFAULT_SYSTEM_PROMPT_TEMPLATE: &str = "TODO";
pub static DEFAULT_USER_PROMPT_TEMPLATE: &str = "TODO";
pub static DEFAULT_RELEVANT_TABLES_TEMPLATE: &str = "TODO";
pub static DEFAULT_SIMILAR_QUERIES_TEMPLATE: &str = "TODO";
pub static DEFAULT_FILTER_DDLS_TEMPLATE: &str = "TODO";

#[derive(PostgresEnum, Eq, PartialEq, Clone)]
pub enum TableFilterType {
    Quick,
    Smart,
}

impl TableFilterType {
    pub fn from_str(val: &str) -> Result<Self, ModelDriverError> {
        match val {
            "quick" => Ok(Self::Quick),
            "smart" => Ok(Self::Smart),
            _ => Err(ModelDriverError::ParseError(format!(
                "unable to parse filter type: {}",
                val
            ))),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TableFilterType::Quick => "quick",
            TableFilterType::Smart => "smart",
        }
    }
}

#[derive(Clone)]
pub struct TextToSqlEngine {
    pub name: String,
    pub schema_name: String,
    pub encoder_model: String,
    pub instruct_model: String,
    pub system_prompt_template: String,
    pub user_prompt_template: String,
    pub relevant_ddls_template: String,
    pub similar_queries_template: String,
    pub filter_ddls_template: String,
    pub filter_type: TableFilterType,
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
        let relevant_ddls_template: String =
            match get_column_optional(&row, "relevant_ddls_template")? {
                Some(v) => v,
                None => DEFAULT_RELEVANT_TABLES_TEMPLATE.to_string(),
            };
        let similar_queries_template: String =
            match get_column_optional(&row, "similar_queries_template")? {
                Some(v) => v,
                None => DEFAULT_SIMILAR_QUERIES_TEMPLATE.to_string(),
            };
        let filter_tables_template: String =
            match get_column_optional(&row, "filter_ddls_template")? {
                Some(v) => v,
                None => DEFAULT_FILTER_DDLS_TEMPLATE.to_string(),
            };

        let name = get_column(&row, "name")?;
        let schema_name = get_column(&row, "schema_name")?;
        let encoder_model = get_column(&row, "encoder_model")?;
        let instruct_model = get_column(&row, "instruct_model")?;
        let filter_type_str = get_column(&row, "filter_type")?;
        let filter_type = TableFilterType::from_str(filter_type_str)?;

        Ok(Self {
            name,
            schema_name,
            encoder_model,
            instruct_model,
            system_prompt_template,
            user_prompt_template,
            relevant_ddls_template,
            similar_queries_template,
            filter_ddls_template: filter_tables_template,
            filter_type,
        })
    }
}
