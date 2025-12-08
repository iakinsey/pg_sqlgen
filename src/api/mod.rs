pub mod internal;
pub mod public;

use pgrx::extension_sql_file;

extension_sql_file!("../../sql-scripts/api.sql", name = "api");
extension_sql_file!("../../sql-scripts/config.sql", name = "config");
extension_sql_file!(
    "../../sql-scripts/engine.sql",
    name = "engine",
    requires = ["model_profile"]
);
extension_sql_file!(
    "../../sql-scripts/internal.sql",
    name = "internal",
    bootstrap
);
extension_sql_file!(
    "../../sql-scripts/model_profile.sql",
    name = "model_profile"
);
extension_sql_file!("../../sql-scripts/profiles.sql", name = "profiles");
extension_sql_file!("../../sql-scripts/metadata.sql", name = "metadata");
extension_sql_file!("../../sql-scripts/certify.sql", name = "certify");
extension_sql_file!("../../sql-scripts/vectors.sql", name = "vectors");
extension_sql_file!(
    "../../sql-scripts/finalized.sql",
    name = "finalized",
    finalize
);
