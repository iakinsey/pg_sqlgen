// Mistral 7b

use hf_hub::{api::sync::Api, Repo};

use crate::types::{
    errors::ModelDriverError,
    structs::{profiles::LocalMistralConfig, ModelProfile},
    traits::driver::{ModelDriver, TextInstructDriver},
};

pub struct LocalMistralDriver {}

impl ModelDriver for LocalMistralDriver {
    const NAME: &'static str = "Local Mistral";
    const DESCRIPTION: &'static str = "A fast, open-weight large language model, runs locally.";
}

impl LocalMistralDriver {
    pub fn new(mistral_config: LocalMistralConfig) -> Result<Self, ModelDriverError> {
        let repo = Repo::with_revision(
            mistral_config.model_name.clone(),
            hf_hub::RepoType::Model,
            mistral_config.revision.clone(),
        );
        let api = Api::new()?.repo(repo);
        let tokenizer_file = api.get(&mistral_config.tokenizer_filename)?;
    }
}

impl TextInstructDriver for LocalMistralDriver {
    fn instruct(&self, system_prompt: &str, user_prompt: &str) -> Result<String, ModelDriverError> {
        unimplemented!()
    }

    fn instruct_many(
        &self,
        system_prompt: &str,
        user_prompts: &[&str],
    ) -> Result<Vec<String>, ModelDriverError> {
        unimplemented!()
    }
}
