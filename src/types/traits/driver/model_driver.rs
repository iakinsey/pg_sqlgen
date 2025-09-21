pub trait ModelDriver {
    const ID: &'static str;
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}
