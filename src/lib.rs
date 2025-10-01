use pgrx::prelude::*;

mod api;
mod drivers;
mod stores;
mod types;
mod utils;

::pgrx::pg_module_magic!(name, version);

extension_sql_file!("../sql-scripts/internal.sql", name = "internal");

#[pg_extern]
fn hello_pg_sqlgen() -> &'static str {
    "Hello, pg_sqlgen"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_pg_sqlgen() {
        assert_eq!("Hello, pg_sqlgen", crate::hello_pg_sqlgen());
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
