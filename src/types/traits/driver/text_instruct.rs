use crate::types::errors::ModelDriverError;

pub trait TextInstructDriver {
    fn instruct(&self, system_prompt: &str, user_prompt: &str) -> Result<String, ModelDriverError>;
    fn instruct_many(
        &self,
        system_prompt: &str,
        user_prompts: &[&str],
    ) -> Result<Vec<String>, ModelDriverError>;
}
