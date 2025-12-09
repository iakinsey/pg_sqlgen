use crate::{
    stores::config_store::{
        ConfigStore, VECTOR_COLUMN_IMPL_DEFAULT, VECTOR_COLUMN_IMPL_KEY,
        VECTOR_COLUMN_IMPL_PGVECTOR,
    },
    types::errors::SqlgenError,
};

pub fn convert_vectors_to_pgvector() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn convert_vectors_to_in_memory() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn guess_vectors_and_convert() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn get_vector_column_type() -> String {
    match ConfigStore::get_config_value(VECTOR_COLUMN_IMPL_KEY) {
        Ok(s) if s == VECTOR_COLUMN_IMPL_PGVECTOR => format!("VECTOR"),
        Ok(s) if s == VECTOR_COLUMN_IMPL_DEFAULT => "FLOAT4[]".into(),
        Err(SqlgenError::ConfigDoesntExist(_)) => "FLOAT4[]".into(),
        _ => "FLOAT4[]".into(),
    }
}
