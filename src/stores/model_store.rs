use pgrx::{spi::SpiTupleTable, IntoDatum, PgBuiltInOids, Spi};
use serde_json::from_str;

use crate::{
    drivers::{LocalBertDriver, OllamaDriver},
    types::{
        errors::ModelDriverError,
        structs::model_profile::{ModelConfig, ModelProfile},
        traits::driver::{ModelDriver, TextEncoderDriver, TextInstructDriver},
    },
};

pub struct ModelStore {}

impl ModelStore {
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
        ]
    }

    pub fn get_model_profile(name: &str) -> Result<ModelProfile, ModelDriverError> {
        let query = "SELECT model_name, driver_name, config::text FROM sqlgen.get_model($1)";

        Spi::connect(|client| {
            let row = client.select(query, None, &[name.into()])?;

            if row.is_empty() {
                return Err(ModelDriverError::ModelDoesntExist(name.to_string()));
            }

            ModelProfile::from_row(row)
        })
    }

    pub fn create_model_profile(
        name: &str,
        config_str: &str,
    ) -> Result<ModelProfile, ModelDriverError> {
        let config: ModelConfig = from_str(config_str)?;
        let profile = ModelProfile {
            name: name.to_string(),
            config: config,
        };
        let query = "SELECT sqlgen.create_model($1, $2);";

        Spi::run_with_args(query, &[name.into(), config_str.into()])?;

        Ok(profile)
    }

    pub fn delete_model_profile(name: &str) -> Result<(), ModelDriverError> {
        Self::get_model_profile(name)?;

        let query = "SELECT model_name, driver_name, config::text FROM sqlgen.get_model($1)";

        Spi::run_with_args(query, &[name.into()])?;

        Ok(())
    }

    pub fn get_text_encoder_model(
        model_name: &str,
    ) -> Result<Box<dyn TextEncoderDriver>, ModelDriverError> {
        let profile = Self::get_model_profile(model_name)?;

        profile.get_text_encoder_model()
    }

    pub fn get_text_instruct_model(
        model_name: &str,
    ) -> Result<Box<dyn TextInstructDriver>, ModelDriverError> {
        let profile = Self::get_model_profile(model_name)?;

        profile.get_text_instruct_model()
    }
}
