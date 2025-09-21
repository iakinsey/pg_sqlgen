pub enum InstructRole {
    System,
    User,
    Assistant
}

pub struct InstructMessage {
    pub role: InstructRole,
    pub message: String,
}