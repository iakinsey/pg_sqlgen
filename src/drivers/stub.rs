use crate::types::{
    errors::SqlgenError,
    structs::{instruct_message::InstructMessage, profiles::StubConfig},
    traits::driver::{ModelDriver, TextEncoderDriver, TextInstructDriver},
};
use async_trait::async_trait;

pub struct StubDriver {
    config: StubConfig,
}

impl StubDriver {
    pub fn new(config: &StubConfig) -> Result<Self, SqlgenError> {
        Ok(Self {
            config: config.clone(),
        })
    }
}

impl ModelDriver for StubDriver {
    const ID: &'static str = "stub";
    const NAME: &'static str = "Stub";
    const DESCRIPTION: &'static str = "A stub model that returns user-specified data.";
}

#[async_trait]
impl TextEncoderDriver for StubDriver {
    async fn dimensions(&self) -> Result<usize, SqlgenError> {
        Ok(self.config.encode_output.len())
    }

    async fn encode(&self, _input: &str) -> Result<Vec<f32>, SqlgenError> {
        Ok(self.config.encode_output.clone())
    }

    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, SqlgenError> {
        Ok(inputs
            .iter()
            .map(|_| self.config.encode_output.clone())
            .collect())
    }
}

#[async_trait]
impl TextInstructDriver for StubDriver {
    async fn get_assistant_response(
        &mut self,
        _messages: Vec<InstructMessage>,
    ) -> Result<String, SqlgenError> {
        Ok(self.config.instruct_output.clone())
    }
}
