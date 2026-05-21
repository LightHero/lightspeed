#![cfg(feature = "mysql")]

use std::sync::OnceLock;

use lightspeed_scheduler::MySqlScheduleRepository;
use lightspeed_test_utils::mysql::new_mysql_db;
use maybe_once::tokio::{Data, MaybeOnceAsync};
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;

mod tests;
mod utils;

/// Concrete repository the shared `tests::*` suite runs against. Each `*_it.rs`
/// binary declares its own alias so generic test code can refer to it as
/// `crate::RepoUnderTest`.
pub type RepoUnderTest = MySqlScheduleRepository;

type SharedFixture = (RepoUnderTest, ContainerAsync<Mysql>);

async fn init() -> SharedFixture {
    let (c3p0, node) = new_mysql_db().await;
    let repo = MySqlScheduleRepository::init(c3p0)
        .await
        .expect("constructor must apply migrations");
    (repo, node)
}

/// Lazily initialises the shared container/pool.
pub async fn data(serial: bool) -> Data<'static, SharedFixture> {
    static DATA: OnceLock<MaybeOnceAsync<SharedFixture>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

/// Returns a freshly-`init`ed repository pointing at the **same** backing
/// storage as the fixture's. For MySQL that means re-running migrations
/// against the shared pool, which must succeed because sqlx tracks applied
/// migrations.
pub async fn start_repo() -> RepoUnderTest {
    let c3p0 = data(false).await.0.c3p0().clone();
    MySqlScheduleRepository::init(c3p0)
        .await
        .expect("init must be idempotent against the same pool")
}
