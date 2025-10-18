use pgrx::Spi;

use crate::{types::errors::StoreError, utils::sql::get_column};

pub struct ConfigStore {}

pub static DEFAULT_ENGINE_CONFIG_KEY: &str = "default_engine";

impl ConfigStore {
    pub fn get_config_value(key: &str) -> Result<String, StoreError> {
        let query = "SELECT sqlgen_internal.get_config_value($1);";

        Spi::connect(|client| {
            let row = client.select(query, None, &[key.into()])?;

            if row.is_empty() {
                return Err(StoreError::ConfigDoesntExist(key.to_string()));
            }

            Ok(get_column(&row, "value")?)
        })
    }

    pub fn set_config_value(key: &str, value: &str) -> Result<(), StoreError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.set_config_value($1, $2);",
            &[key.into(), value.into()],
        )?;

        Ok(())
    }
}
