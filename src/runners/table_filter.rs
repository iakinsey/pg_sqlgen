use crate::types::{errors::StoreError, structs::engine::TextToSqlEngine};
use tera::{Context, Tera};

pub struct TableFilterRunner {}

/*
    Example filter template:

    Filter this list, only select elements that you believe are relevant to the following query:

    {relevant_ddls_block}


*/

impl TableFilterRunner {
    pub fn new(engine: TextToSqlEngine) -> Result<Self, StoreError> {
        let mut tera = Tera::default();

        unimplemented!()
    }
}
