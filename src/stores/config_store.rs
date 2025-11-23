use pgrx::Spi;

use crate::{types::errors::SqlgenError, utils::sql::get_column};

pub struct ConfigStore {}

pub static DEFAULT_ENGINE_CONFIG_KEY: &str = "default_engine";

impl ConfigStore {
    pub fn get_config_value(key: &str) -> Result<String, SqlgenError> {
        let query = r#"
            SELECT v AS value
            FROM (SELECT sqlgen_internal.get_config_value($1) AS v) s
            WHERE v IS NOT NULL;
        "#;

        Spi::connect(|client| {
            let rows = client.select(query, None, &[key.into()])?;

            if rows.is_empty() {
                return Err(SqlgenError::ConfigDoesntExist(key.to_string()));
            }

            let row = rows.first();

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

    pub fn remove_config_value(key: &str) -> Result<(), SqlgenError> {
        Spi::run_with_args(
            "SELECT sqlgen_internal.remove_config_value($1);",
            &[key.into()],
        )?;

        Ok(())
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use crate::{pg_test, stores::config_store::ConfigStore};

    #[pg_test]
    fn test_config_get_set() {
        let key = "test_key";
        let value = "test_value";
        let second_value = "test_value_2";

        ConfigStore::set_config_value(key, value).unwrap();
        let actual_value = ConfigStore::get_config_value(key).unwrap();

        assert_eq!(value, actual_value);

        ConfigStore::set_config_value(key, second_value).unwrap();
        let actual_second_value = ConfigStore::get_config_value(key).unwrap();

        assert_eq!(second_value, actual_second_value);
    }
}
