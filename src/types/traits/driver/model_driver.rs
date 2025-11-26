// Provides identifying information for a model, when querying
// `sqlgen.model_descriptions`.
pub trait ModelDriver {
    const ID: &'static str;
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}
