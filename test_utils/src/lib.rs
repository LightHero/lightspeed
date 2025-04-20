use std::sync::OnceLock;

pub mod mysql;
pub mod pg;
pub mod sqlite;

/// Executes all tests with a single tokio runtime.
/// This allows sharing runtime bounded resources between tests.
pub fn tokio_test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Should create a tokio runtime")
    })
    .block_on(f)
}
