pub mod api;
pub mod internal;

use pgrx::extension_sql_file;

extension_sql_file!("../../sql-scripts/metadata.sql", name = "crawler");
extension_sql_file!(
    "../../sql-scripts/finalized.sql",
    name = "finalized",
    finalize
);
