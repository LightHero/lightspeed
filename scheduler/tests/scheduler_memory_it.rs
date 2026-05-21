//! Integration test binary that runs the shared `tests::*` suite against the
//! in-memory backend. Mirrors `postgres_it.rs` but skips the testcontainer
//! setup — the fixture is just a fresh [`MemoryScheduleRepository`].

use std::sync::OnceLock;

use maybe_once::tokio::{Data, MaybeOnceAsync};
use lightspeed_scheduler::MemoryScheduleRepository;

mod tests;
mod utils;

/// Concrete repository the shared `tests::*` suite runs against. Each `*_it.rs`
/// binary declares its own alias so generic test code can refer to it as
/// `crate::RepoUnderTest`.
pub type RepoUnderTest = MemoryScheduleRepository;

/// One-tuple so shared tests can use `d.0` uniformly across binaries.
type SharedFixture = (RepoUnderTest,);

async fn init() -> SharedFixture {
    (MemoryScheduleRepository::init(),)
}

/// Lazily initialises the shared in-memory fixture.
pub async fn data(serial: bool) -> Data<'static, SharedFixture> {
    static DATA: OnceLock<MaybeOnceAsync<SharedFixture>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

/// Returns a freshly-`init`ed repository. For the in-memory backend there
/// is no shared backing to be idempotent over — this just constructs a new
/// independent instance, which is what "init twice" reduces to here.
pub async fn start_repo() -> RepoUnderTest {
    MemoryScheduleRepository::init()
}
