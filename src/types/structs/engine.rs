use pgrx::spi::SpiTupleTable;

use crate::{
    types::errors::SqlgenError,
    utils::sql::{get_column, get_column_optional},
};

pub static DEFAULT_SYSTEM_PROMPT_TEMPLATE: &str = r#"
You are a helpful SQL generation system. You output valid SQL in the PostgresSQL dialect.
"#;
pub static DEFAULT_USER_PROMPT_TEMPLATE: &str = r#"
Generate SQL for the given request. If the input does not contain enough
information to construct a valid query, return an error indicating that
the information is insufficient. Do not invent tables, columns, or values;
use only what is explicitly provided. Think step-by-step and provide your
reasoning.

Query:
{{user_query}}

{{relevant_ddls_block}}

{{similar_queries_block}}
"#;
pub static DEFAULT_RELEVANT_DDLS_TEMPLATE: &str = r#"
Here are some columns that are possibly relevant to the query.

{{relevant_ddls}}
"#;
// TODO, leave empty for now until similar queries are implemented
pub static DEFAULT_SIMILAR_QUERIES_TEMPLATE: &str = r#"
Here are some example queries from the same database that may or may not be relevant:

{{ similar_queries }}
"#;
pub static DEFAULT_FILTER_DDLS_TEMPLATE: &str = r#"
Given the following query:

{{user_query}}

Filter a list of columns. Select elements relevant to the query. Think
step-by-step and provide your reasoning.

{{relevant_ddls}}
"#;
pub static DEFAULT_SYNTAX_CORRECTION_TEMPLATE: &str = r#"
A query has run into an error when running against PREPARE.
Given the query and error, generate a corrected query so that neither the error nor new errors occur.
Think step-by-step and provide your reasoning.

Query:
{{query}}

Error:
{{error_message}}

{{relevant_ddls_block}}
"#;
pub static DEFAULT_EXPLAIN_QUERY_TEMPLATE: &str = r#"
Given the following query and explain plan, describe what this query does and how it works.
Explain it in simply and succinctly in a a 1-2 paragraph summary. Suggest any optimizations.
Think step-by-step and provide your reasoning.

Query: 
{{sql_query}}

Explain:
{{explain}}
"#;

pub static DEFAULT_JUDGE_QUERY_TEMPLATE: &str = r#"
You are an impartial judge of PostgreSQL queries. Given an SQL query and related metadata,
determine if the query sufficiently reflects the language query provided. If you think the
SQL query does not adequately represent the language query provided, then provide a counter
example. Think step-by-step and provide your reasoning.

Language query:
{{user_query}}

SQL Query:
{{sql_query}}

Explain:
{{explain}}

{{relevant_ddls_block}}

{{similar_queries_block}}
"#;

// Type of filtering used by `DDLFilterRunner`.
#[derive(Eq, PartialEq, Clone)]
pub enum TableFilterType {
    Quick,
    Smart,
}

impl TableFilterType {
    pub fn from_str(val: &str) -> Result<Self, SqlgenError> {
        match val {
            "quick" => Ok(Self::Quick),
            "smart" => Ok(Self::Smart),
            _ => Err(SqlgenError::ParseError(format!(
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

// Contains all of the relevant prompt templates and metadata required to
// construct entities in the `runners` module. State management for this struct
// is handled via `EngineStore`.
#[derive(Clone)]
pub struct TextToSqlEngine {
    pub name: String,
    #[allow(dead_code)]
    pub schema_names: Vec<String>,
    pub encoder_model: String,
    pub instruct_model: String,
    pub system_prompt_template: String,
    pub user_prompt_template: String,
    pub relevant_ddls_template: String,
    pub similar_queries_template: String,
    pub filter_ddls_template: String,
    pub syntax_correction_template: String,
    pub explain_query_template: String,
    pub judge_query_template: String,
    pub filter_type: TableFilterType,
    pub column_filter_limit: i32,
    pub error_correction_rounds: i32,
}

impl TextToSqlEngine {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, SqlgenError> {
        let system_prompt_template: String =
            match get_column_optional(&row, "system_prompt_template")? {
                Some(v) => v,
                None => DEFAULT_SYSTEM_PROMPT_TEMPLATE.to_string(),
            };
        let user_prompt_template: String = match get_column_optional(&row, "user_prompt_template")?
        {
            Some(v) => v,
            None => DEFAULT_USER_PROMPT_TEMPLATE.to_string(),
        };
        let relevant_ddls_template: String =
            match get_column_optional(&row, "relevant_ddls_template")? {
                Some(v) => v,
                None => DEFAULT_RELEVANT_DDLS_TEMPLATE.to_string(),
            };
        let similar_queries_template: String =
            match get_column_optional(&row, "similar_queries_template")? {
                Some(v) => v,
                None => DEFAULT_SIMILAR_QUERIES_TEMPLATE.to_string(),
            };
        let filter_ddls_template: String = match get_column_optional(&row, "filter_ddls_template")?
        {
            Some(v) => v,
            None => DEFAULT_FILTER_DDLS_TEMPLATE.to_string(),
        };
        let syntax_correction_template: String =
            match get_column_optional(&row, "syntax_correction_template")? {
                Some(v) => v,
                None => DEFAULT_SYNTAX_CORRECTION_TEMPLATE.to_string(),
            };
        let explain_query_template: String =
            match get_column_optional(&row, "explain_query_template")? {
                Some(v) => v,
                None => DEFAULT_EXPLAIN_QUERY_TEMPLATE.to_string(),
            };
        let judge_query_template: String = match get_column_optional(&row, "judge_query_template")?
        {
            Some(v) => v,
            None => DEFAULT_JUDGE_QUERY_TEMPLATE.to_string(),
        };

        let name = get_column(&row, "engine_name")?;
        let schema_names = get_column(&row, "schema_names")?;
        let encoder_model = get_column(&row, "encoder_model")?;
        let instruct_model = get_column(&row, "instruct_model")?;
        let filter_type_str = get_column(&row, "table_filter_type")?;
        let filter_type = TableFilterType::from_str(filter_type_str)?;
        let column_filter_limit = get_column(&row, "column_filter_limit")?;
        let error_correction_rounds = get_column(&row, "error_correction_rounds")?;

        Ok(Self {
            name,
            schema_names,
            encoder_model,
            instruct_model,
            system_prompt_template,
            user_prompt_template,
            relevant_ddls_template,
            similar_queries_template,
            filter_ddls_template,
            syntax_correction_template,
            explain_query_template,
            judge_query_template,
            filter_type,
            column_filter_limit,
            error_correction_rounds,
        })
    }

    pub fn get_generate_system_prompt(&self) -> Result<String, SqlgenError> {
        // TODO remove
        Ok(self.system_prompt_template.clone())
    }
}
