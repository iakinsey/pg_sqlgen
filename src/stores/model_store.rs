use pgrx::{pg_schema, Spi};
use serde_json::from_str;

use crate::{
    drivers::{LocalBertDriver, OllamaDriver, StubDriver},
    types::{
        errors::SqlgenError,
        structs::model_profile::{ModelConfig, ModelProfile},
        traits::driver::{ModelDriver, TextEncoderDriver, TextInstructDriver},
    },
};

pub struct ModelStore {}

// TODO add get_model_capabilities, provide a way to check if a model could be instruct/encoder/etc
impl ModelStore {
    pub fn get_model_profile(name: &str) -> Result<ModelProfile, SqlgenError> {
        let query = "SELECT model_name, config::TEXT AS config FROM sqlgen.get_model($1)";
        let result = Spi::connect(|client| {
            let row = client.select(query, None, &[name.into()])?;

            if row.is_empty() {
                return Err(SqlgenError::ModelDoesntExist(name.to_string()));
            }

            Ok(ModelProfile::from_row(row.first())?)
        });

        Ok(result?)
    }

    pub fn create_model_profile(name: &str, config_str: &str) -> Result<ModelProfile, SqlgenError> {
        let config: ModelConfig = from_str(config_str)?;
        let profile = ModelProfile {
            name: name.to_string(),
            config: config,
        };
        let query = "SELECT sqlgen.create_model($1, $2::JSONB);";

        Spi::run_with_args(query, &[name.into(), config_str.into()])?;

        Ok(profile)
    }

    pub fn delete_model_profile(name: &str) -> Result<(), SqlgenError> {
        Self::get_model_profile(name)?;

        let query = "SELECT sqlgen.delete_model($1)";

        Spi::run_with_args(query, &[name.into()])?;

        Ok(())
    }

    pub fn get_text_encoder_model(
        model_name: &str,
    ) -> Result<Box<dyn TextEncoderDriver>, SqlgenError> {
        let profile = Self::get_model_profile(model_name)?;

        Ok(profile.get_text_encoder_model()?)
    }

    pub fn get_text_instruct_model(
        model_name: &str,
    ) -> Result<Box<dyn TextInstructDriver>, SqlgenError> {
        let profile = Self::get_model_profile(model_name)?;

        Ok(profile.get_text_instruct_model()?)
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    use crate::stores::model_store::ModelStore;

    #[pg_test]
    fn test_model_store_crud() {
        let model_name = "test_model";
        let created_profile = ModelStore::create_model_profile(model_name, "{}").unwrap();
        let got_profile = ModelStore::get_model_profile(model_name).unwrap();

        assert_eq!(created_profile.name, got_profile.name);
        assert_eq!(created_profile.config, got_profile.config);
    }
}
