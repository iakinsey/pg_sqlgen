use async_trait::async_trait;

use crate::types::{errors::ModelDriverError, structs::instruct_message::InstructMessage};

#[async_trait]
pub trait TextInstructDriver: Send + Sync {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, ModelDriverError>;
}
