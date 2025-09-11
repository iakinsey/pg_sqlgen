pub trait ModelDriver {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;

    fn initialize() -> Result<(), ModelDriverError>;
    fn destroy() -> Result<(), ModelDriverError>;
}
