#![cfg(feature = "sqlite")]

use std::sync::OnceLock;

use lightspeed_scheduler::SqliteScheduleRepository;
use lightspeed_test_utils::sqlite::new_sqlite_db;
use maybe_once::tokio::{Data, MaybeOnceAsync};

mod tests;
mod utils;

/// Concrete repository the shared `tests::*` suite runs against. Each `*_it.rs`
/// binary declares its own alias so generic test code can refer to it as
/// `crate::RepoUnderTest`.
pub type RepoUnderTest = SqliteScheduleRepository;

/// One-tuple so shared tests can use `d.0` uniformly across binaries.
type SharedFixture = (RepoUnderTest,);

async fn init() -> SharedFixture {
    let c3p0 = new_sqlite_db().await;
    let repo = SqliteScheduleRepository::init(c3p0).await.expect("constructor must apply migrations");
    (repo,)
}

/// Lazily initialises the shared in-memory pool.
pub async fn data(serial: bool) -> Data<'static, SharedFixture> {
    static DATA: OnceLock<MaybeOnceAsync<SharedFixture>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}

/// Returns a freshly-`init`ed repository pointing at the **same** backing
/// storage as the fixture's. Re-running migrations against the shared pool
/// must succeed because sqlx tracks applied migrations.
pub async fn start_repo() -> RepoUnderTest {
    let c3p0 = data(false).await.0.c3p0().clone();
    SqliteScheduleRepository::init(c3p0).await.expect("init must be idempotent against the same pool")
}
