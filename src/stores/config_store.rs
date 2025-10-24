use pgrx::{pg_schema, Spi};

use crate::{types::errors::SqlgenError, utils::sql::get_column};

pub struct ConfigStore {}

pub static DEFAULT_ENGINE_CONFIG_KEY: &str = "default_engine";

impl ConfigStore {
    pub fn get_config_value(key: &str) -> Result<String, SqlgenError> {
        let query = "SELECT sqlgen_internal.get_config_value($1);";

        Spi::connect(|client| {
            let row = client.select(query, None, &[key.into()])?;

            if row.is_empty() {
                return Err(SqlgenError::ConfigDoesntExist(key.to_string()));
            }

            Ok(get_column(&row, "value")?)
        })
    }

    pub fn set_config_value(key: &str, value: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.set_config_value($1, $2);",
            &[key.into(), value.into()],
        )?;

        Ok(())
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::Spi;

    use crate::pg_test;

    #[pg_test]
    fn test_config_get_set() {
        // TODO test
        unimplemented!()
    }
}
