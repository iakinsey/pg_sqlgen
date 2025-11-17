mod local_bert;
mod local_phi_instruct;
mod ollama;
mod openai_completions;
mod openai_embeddings;
mod stub;

use crate::types::traits::driver::ModelDriver;
pub use local_bert::*;
pub use local_phi_instruct::*;
pub use ollama::*;
pub use openai_completions::*;
pub use openai_embeddings::*;
pub use stub::*;

pub fn get_model_descriptions() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            LocalBertDriver::ID,
            LocalBertDriver::NAME,
            LocalBertDriver::DESCRIPTION,
        ),
        (
            OllamaDriver::ID,
            OllamaDriver::NAME,
            OllamaDriver::DESCRIPTION,
        ),
        (StubDriver::ID, StubDriver::NAME, StubDriver::DESCRIPTION),
    ]
}
