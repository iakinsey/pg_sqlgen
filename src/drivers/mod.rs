mod ollama;
mod openai_completions;
mod openai_embeddings;
mod stub;

use crate::types::traits::driver::ModelDriver;
pub use ollama::*;
pub use openai_completions::*;
pub use openai_embeddings::*;
pub use stub::*;

pub fn get_model_descriptions() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            OpenAICompletionsDriver::ID,
            OpenAICompletionsDriver::NAME,
            OpenAICompletionsDriver::DESCRIPTION,
        ),
        (
            OllamaDriver::ID,
            OllamaDriver::NAME,
            OllamaDriver::DESCRIPTION,
        ),
        (StubDriver::ID, StubDriver::NAME, StubDriver::DESCRIPTION),
    ]
}
