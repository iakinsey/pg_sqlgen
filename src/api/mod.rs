pub mod api;

pub use api::*;

use pgrx::extension_sql_file;

extension_sql_file!("../../sql/metadata.sql", name = "crawler");
