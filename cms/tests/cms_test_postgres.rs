#![cfg(feature = "postgres")]

use std::sync::OnceLock;
use std::time::Duration;

use c3p0::postgres::PgC3p0Pool;
use c3p0::postgres::deadpool::{self, Runtime};
use c3p0::postgres::tokio_postgres::NoTls;
use lightspeed_test_utils::pg::new_pg_db;
use maybe_once::tokio::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::postgres::PostgresCmsRepositoryManager;
use lightspeed_cms::LsCmsModule;
use lightspeed_core::module::LsModule;

use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type RepoManager = PostgresCmsRepositoryManager;

pub type MaybeType = (LsCmsModule<RepoManager>, ContainerAsync<Postgres>);

async fn init() -> MaybeType {
    let (_c3p0, node) = new_pg_db().await;

    let mut config = c3p0::postgres::deadpool::postgres::Config {
        user: Some("postgres".to_owned()),
        password: Some("postgres".to_owned()),
        dbname: Some("postgres".to_owned()),
        host: Some("127.0.0.1".to_string()),
        port: Some(node.get_host_port_ipv4(5432).await.unwrap()),
        ..Default::default()
    };

    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let c3p0 = PgC3p0Pool::new(config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap());

    let repo_manager = RepoManager::new(c3p0);

    let cms_config = CmsConfig::default();

    let mut cms_module = LsCmsModule::new(repo_manager, cms_config);
    cms_module.start().await.unwrap();

    (cms_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
