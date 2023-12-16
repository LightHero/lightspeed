use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::*;
use maybe_single::tokio::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::pg::PgCmsRepositoryManager;
use lightspeed_cms::LsCmsModule;
use lightspeed_core::module::LsModule;
use once_cell::sync::OnceCell;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::clients::Cli;
use testcontainers::testcontainers::Container;

mod tests;

pub type RepoManager = PgCmsRepositoryManager;

pub type MaybeType = (LsCmsModule<RepoManager>, Container<'static, Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();

    let node = DOCKER.get_or_init(Cli::default).run(Postgres::default());

    let options = PgConnectOptions::new()
    .username("postgres")
    .password("postgres")
    .database("postgres")
    .host("127.0.0.1")
    .port(node.get_host_port_ipv4(5432));

    let pool = PgPool::connect_with(options).await.unwrap();

    let c3p0 = SqlxPgC3p0Pool::new(pool);

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
