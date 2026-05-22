#![cfg(feature = "mysql")]

use std::sync::OnceLock;

use maybe_once::tokio::*;

use lightspeed_account_management::LsAMModule;
use lightspeed_account_management::config::AMConfig;
use lightspeed_account_management::repository::mysql::MyAMSqlRepositoryManager;
use lightspeed_core::module::LsModule;
use lightspeed_test_utils::mysql::new_mysql_db;
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type RepoManager = MyAMSqlRepositoryManager;

pub type MaybeType = (LsAMModule<RepoManager>, ContainerAsync<Mysql>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_mysql_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    // Argon2 spec minimum (memory=8 KiB, t=1, p=1) — fast for tests.
    let auth_config =
        AMConfig { argon2_memory_kib: 8, argon2_iterations: 1, argon2_parallelism: 1, ..Default::default() };

    let mut auth_module = LsAMModule::new(repo_manager, auth_config).unwrap();
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
