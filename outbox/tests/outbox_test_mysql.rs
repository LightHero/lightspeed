#![cfg(feature = "mysql")]

use std::sync::OnceLock;

use lightspeed_outbox::LsOutboxModule;
use lightspeed_outbox::config::OutboxConfig;
use lightspeed_outbox::repository::mysql::MySqlOutboxRepositoryManager;
use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use lightspeed_test_utils::mysql::new_mysql_db;
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type RepoManager = MySqlOutboxRepositoryManager;

pub type MaybeType = (LsOutboxModule<RepoManager>, ContainerAsync<Mysql>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_mysql_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let auth_config = OutboxConfig { ..Default::default() };

    let mut auth_module = LsOutboxModule::new(repo_manager, auth_config);
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
