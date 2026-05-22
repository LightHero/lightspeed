#![cfg(feature = "postgres")]

use std::sync::OnceLock;

use lightspeed_scheduler::PgScheduleRepository;
use lightspeed_test_utils::pg::new_pg_db;
use maybe_once::tokio::{Data, MaybeOnceAsync};
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;

mod tests;
mod utils;

/// Concrete repository the shared `tests::*` suite runs against. Each `*_it.rs`
/// binary declares its own alias so generic test code can refer to it as
/// `crate::RepoUnderTest`.
pub type RepoUnderTest = PgScheduleRepository;

type SharedFixture = (RepoUnderTest, ContainerAsync<Postgres>);

async fn init() -> SharedFixture {
    let (c3p0, node) = new_pg_db().await;
    let repo = PgScheduleRepository::init(c3p0).await.expect("constructor must apply migrations");
    (repo, node)
}

/// Lazily initialises the shared container/pool.
pub async fn data(serial: bool) -> Data<'static, SharedFixture> {
    static DATA: OnceLock<MaybeOnceAsync<SharedFixture>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}

/// Returns a freshly-`init`ed repository pointing at the **same** backing
/// storage as the fixture's. For Postgres that means re-running migrations
/// against the shared pool, which must succeed because sqlx tracks applied
/// migrations.
pub async fn start_repo() -> RepoUnderTest {
    let c3p0 = data(false).await.0.c3p0().clone();
    PgScheduleRepository::init(c3p0).await.expect("init must be idempotent against the same pool")
}
