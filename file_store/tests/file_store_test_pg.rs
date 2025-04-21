#![cfg(feature = "postgres")]

use std::sync::OnceLock;

use maybe_once::tokio::*;

use lightspeed_core::module::LsModule;
use lightspeed_file_store::LsFileStoreModule;
use lightspeed_file_store::config::FileStoreConfig;
use lightspeed_file_store::repository::db::pg::PgFileStoreRepositoryManager;
use opendal::services::Fs;
use opendal::Operator;
use test_utils::pg::new_pg_db;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type RepoManager = PgFileStoreRepositoryManager;

pub type MaybeType = (LsFileStoreModule<RepoManager>, ContainerAsync<Postgres>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_pg_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let mut file_store_config = FileStoreConfig::default();
    file_store_config.repositories.insert("REPO_ONE".to_owned(), Operator::new(Fs::default().root("../target/repo_one")).unwrap().finish().into());
    file_store_config.repositories.insert("REPO_TWO".to_owned(), Operator::new(Fs::default().root("../target/repo_two")).unwrap().finish().into());

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
