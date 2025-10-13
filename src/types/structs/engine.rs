pub static DEFAULT_GENERATE_PROMPT: &str = "TODO";
pub static DEFAULT_FILTER_PROMPT: &str = "TODO";

pub struct TextToSqlEngine {
    pub name: String,
    pub db_schema: String,
    pub encoder_model: String,
    pub instruct_model: String,
    pub generate_prompt: String,
    pub filter_prompt: String,
}

impl TextToSqlEngine {
    pub fn from_row() {
        // TODO
    }
}
