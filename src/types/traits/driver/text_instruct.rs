use async_trait::async_trait;

use crate::types::{errors::SqlgenError, structs::instruct_message::InstructMessage};

// Interface for chat interactions with models.
#[async_trait]
pub trait TextInstructDriver: Send + Sync {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, SqlgenError>;
}
