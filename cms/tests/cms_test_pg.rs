use c3p0::postgres::deadpool;
use c3p0::postgres::tokio_postgres::*;
use c3p0::postgres::*;
use maybe_single::*;
use testcontainers::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::pg::PgCmsRepositoryManager;
use lightspeed_cms::CmsModule;
use lightspeed_core::module::Module;
use once_cell::sync::OnceCell;
use tokio::time::Duration;

mod tests;

pub type RepoManager = PgCmsRepositoryManager;

pub type MaybeType = (CmsModule<RepoManager>, Container<'static, clients::Cli, images::postgres::Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();

    let node = DOCKER.get_or_init(|| clients::Cli::default()).run(images::postgres::Postgres::default());

    let mut config = deadpool::postgres::Config::default();
    config.user = Some("postgres".to_owned());
    config.password = Some("postgres".to_owned());
    config.dbname = Some("postgres".to_owned());
    config.host = Some(format!("127.0.0.1"));
    config.port = Some(node.get_host_port(5432).unwrap());
    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let c3p0 = PgC3p0Pool::new(config.create_pool(NoTls).unwrap());

    let repo_manager = RepoManager::new(c3p0);

    let cms_config = CmsConfig::build();

    let mut cms_module = CmsModule::new(repo_manager, cms_config);
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
