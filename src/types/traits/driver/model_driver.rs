use crate::types::{errors::ModelDriverError, structs::ModelProfile};

pub trait ModelDriver {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;

    fn initialize(profile: ModelProfile) -> Result<(), ModelDriverError>;
    fn destroy(profile: ModelProfile) -> Result<(), ModelDriverError>;
}
