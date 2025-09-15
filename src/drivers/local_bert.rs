// BERT

use std::fs::read_to_string;

use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo};
use serde_json::from_str;
use tokenizers::Tokenizer;

use crate::{
    types::{
        errors::ModelDriverError,
        structs::{model_profile::ModelProfile, profiles::LocalBertProfile},
        traits::driver::{ModelDriver, TextEncoderDriver},
    },
    utils::math::l2_norm,
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
        let tokens = self.tokenizer.encode_batch(inputs.to_vec(), true)?;
        let device = self.profile.get_device()?;
        let (token_ids, attention_mask) = tokens.iter().try_fold(
            (Vec::new(), Vec::new()),
            |(mut token_ids, mut attention_mask), t| {
                let ids_tensor = Tensor::new(t.get_ids().to_vec().as_slice(), &device)?;
                let attn_tensor = Tensor::new(t.get_attention_mask().to_vec().as_slice(), &device)?;

                token_ids.push(ids_tensor);
                attention_mask.push(attn_tensor);

                Ok::<(Vec<Tensor>, Vec<Tensor>), ModelDriverError>((token_ids, attention_mask))
            },
        )?;

        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;
        let token_type_ids = token_ids.zeros_like()?;
        let embeddings = self
            .model
            .forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        let (_, n_tokens, _) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
        let embeddings = l2_norm(&embeddings)?;

        (0..inputs.len())
            .map(|i| Ok(embeddings.get(i)?.to_vec1::<f32>()?))
            .collect()
    }
}
