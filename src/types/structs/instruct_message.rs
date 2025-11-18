use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum InstructRole {
    System,
    User,
    Assistant,
}

impl InstructRole {
    pub fn to_string(&self) -> String {
        match self {
            InstructRole::System => "system",
            InstructRole::User => "user",
            InstructRole::Assistant => "assistant",
        }
        .to_string()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InstructMessage {
    pub role: InstructRole,
    pub message: String,
}
