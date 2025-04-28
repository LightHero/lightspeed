#![cfg(feature = "sqlx_sqlite")]

use std::sync::OnceLock;

use lightspeed_file_store::LsFileStoreModule;
use lightspeed_file_store::repository::db::sqlx_sqlite::SqliteFileStoreRepositoryManager;
use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use lightspeed_test_utils::sqlite::new_sqlite_db;
use tests::get_config;

mod tests;

pub type RepoManager = SqliteFileStoreRepositoryManager;

pub type MaybeType = (LsFileStoreModule<RepoManager>, ());

async fn init() -> MaybeType {
    let c3p0 = new_sqlite_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let file_store_config = get_config();

    let mut file_store_module = LsFileStoreModule::new(repo_manager, file_store_config).unwrap();
    {
        file_store_module.start().await.unwrap();
    }

    (file_store_module, ())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
