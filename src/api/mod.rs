pub mod api;

pub use api::*;

use pgrx::extension_sql_file;

extension_sql_file!("../../sql-scripts/metadata.sql", name = "crawler");
