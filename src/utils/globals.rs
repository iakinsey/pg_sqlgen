use std::sync::OnceLock;
use tokio::runtime::Runtime;

static ASYNC_RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Global tokio async runtime.
pub fn get_runtime() -> &'static Runtime {
    ASYNC_RUNTIME.get_or_init(|| Runtime::new().unwrap())
}
