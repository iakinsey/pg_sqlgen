use pgrx::{spi::SpiTupleTable, PostgresEnum};
use tera::{Context, Tera};

use crate::{
    types::errors::SqlgenError,
    utils::sql::{get_column, get_column_optional},
};

pub static SYSTEM_PROMPT_TEMPLATE_KEY: &str = "system";
pub static OUTPUT_FORMAT_VAR_KEY: &str = "output_format_description";
pub static OUTPUT_FORMAT_DESCRIPTION: &str = r#"
You only respond as a json dictionary with the following keys:
 - query (string) : The generated SQL query.
 - error (string) : Optional, in the event you run into a problem generating a query, explain the failure here. 
"#;
pub static EXPLAIN_OUTPUT_FORMAT_DESCRIPTION: &str = r#"
Only respond as a json dictionary with the following keys:
 - text (string) : Query description format.
 - error (string) : Optional, in the event you run into a problem generating a query, explain the failure here. 
"#;
pub static DEFAULT_SYSTEM_PROMPT_TEMPLATE: &str = r#"
You are a helpful SQL generation system. You output valid SQL in the PostgresSQL dialect.

Output in the following format:
{{output_format_description}}
"#;
pub static DEFAULT_USER_PROMPT_TEMPLATE: &str = r#"
Generate SQL from the following query:

{{user_query}}

{{relevant_ddls_block}}

{{similar_queries_block}}
"#;
pub static DEFAULT_RELEVANT_DDLS_TEMPLATE: &str = r#"
Here are some DDLs that are possibly relevant to the query.

{{relevant_ddls}}
"#;
// TODO, leave empty for now until similar queries are implemented
pub static DEFAULT_SIMILAR_QUERIES_TEMPLATE: &str = "";
pub static DEFAULT_FILTER_DDLS_TEMPLATE: &str = r#"
Given the following query:

{{user_query}}

Filter this list of DDLs. Select elements relevant to the query.
Respond only by returning a filtered list of elements from this
list and nothing else:

{{relevant_ddls}}
"#;
pub static DEFAULT_SYNTAX_CORRECTION_TEMPLATE: &str = r#"
A query has run into an error when running against PREPARE.
Given the query and error, generate a corrected query so that neither the error nor new errors occur.

Query:
{{query}}

Error:
{{error_message}}

"#;
pub static DEFAULT_EXPLAIN_QUERY_TEMPLATE: &str = r#"
Given the following query and explain plan, describe what this query does and how it works.
Explain it in simply and succinctly in a a 1-2 paragraph summary. Suggest any optimizations.

{{output_format_description}}

Query: 
{{sql_query}}

Explain:
{{explain}}
"#;

// Type of filtering used by `DDLFilterRunner`.
#[derive(PostgresEnum, Eq, PartialEq, Clone)]
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
    pub schema_name: String,
    pub encoder_model: String,
    pub instruct_model: String,
    pub system_prompt_template: String,
    pub user_prompt_template: String,
    pub relevant_ddls_template: String,
    pub similar_queries_template: String,
    pub filter_ddls_template: String,
    pub syntax_correction_template: String,
    pub explain_query_template: String,
    pub filter_type: TableFilterType,
    pub ddl_prompt_limit: i32,
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
        let name = get_column(&row, "engine_name")?;
        let schema_name = get_column(&row, "schema_name")?;
        let encoder_model = get_column(&row, "encoder_model")?;
        let instruct_model = get_column(&row, "instruct_model")?;
        let filter_type_str = get_column(&row, "table_filter_type")?;
        let filter_type = TableFilterType::from_str(filter_type_str)?;
        let ddl_prompt_limit = get_column(&row, "ddl_prompt_limit")?;

        Ok(Self {
            name,
            schema_name,
            encoder_model,
            instruct_model,
            system_prompt_template,
            user_prompt_template,
            relevant_ddls_template,
            similar_queries_template,
            filter_ddls_template,
            syntax_correction_template,
            explain_query_template,
            filter_type,
            ddl_prompt_limit,
        })
    }

    pub fn get_generate_system_prompt(&self) -> Result<String, SqlgenError> {
        let mut tera = Tera::default();
        let mut sys_ctx = Context::new();

        tera.add_raw_template(SYSTEM_PROMPT_TEMPLATE_KEY, &self.system_prompt_template)?;
        sys_ctx.insert(OUTPUT_FORMAT_VAR_KEY, OUTPUT_FORMAT_DESCRIPTION);

        Ok(tera.render(SYSTEM_PROMPT_TEMPLATE_KEY, &sys_ctx)?)
    }
}
