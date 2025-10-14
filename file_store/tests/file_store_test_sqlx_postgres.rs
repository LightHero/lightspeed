#![cfg(feature = "sqlx_postgres")]

use std::sync::OnceLock;

use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use lightspeed_file_store::LsFileStoreModule;
use lightspeed_file_store::repository::db::postgres::PgFileStoreRepositoryManager;
use lightspeed_test_utils::pg::new_pg_db;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;
use tests::get_config;

mod tests;

pub type RepoManager = PgFileStoreRepositoryManager;

pub type MaybeType = (LsFileStoreModule<RepoManager>, ContainerAsync<Postgres>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_pg_db().await;

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
