use c3p0::postgres::deadpool;
use c3p0::postgres::deadpool::Runtime;
use c3p0::postgres::tokio_postgres::*;
use c3p0::postgres::*;
use maybe_single::nio::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::pg::PgCmsRepositoryManager;
use lightspeed_cms::LsCmsModule;
use lightspeed_core::module::LsModule;
use once_cell::sync::OnceCell;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::clients::Cli;
use testcontainers::testcontainers::Container;
use tokio::time::Duration;

mod tests;

pub type RepoManager = PgCmsRepositoryManager;

pub type MaybeType = (LsCmsModule<RepoManager>, Container<'static, Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();

    let node = DOCKER.get_or_init(Cli::default).run(Postgres::default());

    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));

    let config = deadpool::postgres::Config {
        user: Some("postgres".to_owned()),
        password: Some("postgres".to_owned()),
        dbname: Some("postgres".to_owned()),
        host: Some("127.0.0.1".to_string()),
        port: Some(node.get_host_port_ipv4(5432)),
        pool: Some(pool_config),
        ..Default::default()
    };

    let c3p0 = PgC3p0Pool::new(config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap());

    let repo_manager = RepoManager::new(c3p0);

    let cms_config = CmsConfig::default();

    let mut cms_module = LsCmsModule::new(repo_manager, cms_config);
    cms_module.start().await.unwrap();

    (cms_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init()))).data(serial).await
}

pub fn test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Should create a tokio runtime")
    })
    .block_on(f)
}
