use crate::types::{errors::ModelDriverError, structs::instruct_message::InstructMessage};

pub trait TextInstructDriver {
    fn get_assistant_response(&mut self, messages: Vec<InstructMessage>) -> Result<String, ModelDriverError>;
}
