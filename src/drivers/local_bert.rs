// BERT

use std::fs::read_to_string;

use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo};
use serde_json::from_str;
use tokenizers::Tokenizer;

use crate::types::{
    errors::ModelDriverError,
    structs::{model_profile::ModelProfile, profiles::LocalBertProfile},
    traits::driver::{ModelDriver, TextEncoderDriver},
};

pub struct LocalBertDriver {
    profile: LocalBertProfile,
    model: BertModel,
    tokenizer: Tokenizer,
}

impl LocalBertDriver {
    pub fn new(profile: ModelProfile) -> Result<Self, ModelDriverError> {
        let profile = LocalBertProfile::from_model_profile(profile)?;
        let param_profile = profile.clone();
        let device = profile.get_device()?;
        let repo = Repo::with_revision(
            profile.model_name,
            hf_hub::RepoType::Model,
            profile.revision,
        );
        let api = Api::new()?.repo(repo);
        let config_file = api.get(&profile.config_filename)?;
        let tokenizer_file = api.get(&profile.tokenizer_filename)?;
        let weights_file = api.get(&profile.weights_filename)?;
        let config_json = read_to_string(config_file)?;
        let config: Config = from_str(&config_json)?;
        let tokenizer = Tokenizer::from_file(tokenizer_file)?;
        let var_builder =
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_file], DTYPE, &device)? };
        let model = BertModel::load(var_builder, &config)?;

        Ok(LocalBertDriver {
            profile: param_profile,
            model,
            tokenizer,
        })
    }
}

impl ModelDriver for LocalBertDriver {
    const NAME: &'static str = "Local BERT";
    const DESCRIPTION: &'static str = "A classic and lightweight text encoder that runs locally.";

    fn initialize(profile: ModelProfile) -> Result<(), ModelDriverError> {
        // Run the constructor in order to initiate downloading the model
        Self::new(profile)?;

        Ok(())
    }

    fn destroy(profile: ModelProfile) -> Result<(), ModelDriverError> {
        unimplemented!();
    }
}
impl TextEncoderDriver for LocalBertDriver {
    fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError> {
        unimplemented!();
    }

    fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, ModelDriverError> {
        unimplemented!();
    }
}
