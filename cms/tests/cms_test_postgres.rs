#![cfg(feature = "postgres")]

use std::sync::OnceLock;

use lightspeed_test_utils::pg::new_pg_db;
use maybe_once::tokio::*;

use lightspeed_cms::LsCmsModule;
use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::postgres::PostgresCmsRepositoryManager;
use lightspeed_core::module::LsModule;

use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type RepoManager = PostgresCmsRepositoryManager;

pub type MaybeType = (LsCmsModule<RepoManager>, ContainerAsync<Postgres>);

async fn init() -> MaybeType {
    let (c3p0, node) = new_pg_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

  let cms_config = CmsConfig::default();

    let mut cms_module = LsCmsModule::new(repo_manager, cms_config);
    cms_module.start().await.unwrap();

    (cms_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
