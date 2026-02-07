#![cfg(feature = "sqlite")]

use std::sync::OnceLock;

use lightspeed_outbox::{LsOutboxModule, config::OutboxConfig, repository::sqlite::SqliteOutboxRepositoryManager};
use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use lightspeed_test_utils::sqlite::new_sqlite_db;

mod tests;

pub const DB_TYPE: &str = "sqlite";

pub type RepoManager = SqliteOutboxRepositoryManager;

pub type MaybeType = (LsOutboxModule<RepoManager>, ());

async fn init() -> MaybeType {
    let c3p0 = new_sqlite_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let auth_config = OutboxConfig { ..Default::default() };

    let mut auth_module = LsOutboxModule::new(repo_manager, auth_config);
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, ())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
