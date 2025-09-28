use pgrx::extension_sql_file;

extension_sql_file!("../../../sql/crawler.sql", name = "crawler");

pub fn get_db_metadata(database: String) {}
