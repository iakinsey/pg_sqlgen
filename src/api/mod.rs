pub mod api;
pub mod internal;

use pgrx::extension_sql_file;

extension_sql_file!("../../sql-scripts/engine.sql", name = "engine");
extension_sql_file!("../../sql-scripts/internal.sql", name = "internal");
extension_sql_file!(
    "../../sql-scripts/model_profile.sql",
    name = "model_profile"
);
extension_sql_file!("../../sql-scripts/metadata.sql", name = "metadata");
extension_sql_file!(
    "../../sql-scripts/finalized.sql",
    name = "finalized",
    finalize
);
