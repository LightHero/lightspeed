#![cfg(feature = "mysql")]

use std::sync::OnceLock;

use lightspeed_file_store::LsFileStoreModule;
use lightspeed_file_store::repository::db::mysql::MySqlFileStoreRepositoryManager;
use lightspeed_test_utils::mysql::new_mysql_db;
use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;

use crate::tests::get_config;

mod tests;

pub type RepoManager = MySqlFileStoreRepositoryManager;

pub type MaybeType = (LsFileStoreModule<RepoManager>, ContainerAsync<Mysql>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_mysql_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let file_store_config = get_config();

    let mut file_store_module = LsFileStoreModule::new(repo_manager, file_store_config).unwrap();
    {
        file_store_module.start().await.unwrap();
    }

    (file_store_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
