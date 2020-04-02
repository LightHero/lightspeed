use c3p0::pg_async::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0::pg_async::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};
use testcontainers::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::pg::PgCmsRepositoryManager;
use lightspeed_cms::CmsModule;
use lightspeed_core::module::Module;

mod tests;

pub type RepoManager = PgCmsRepositoryManager;

pub type MaybeType = (
    CmsModule<RepoManager>,
    Container<'static, clients::Cli, images::postgres::Postgres>,
);

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

fn init() -> MaybeType {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();
    let c3p0 = PgC3p0PoolAsync::new(pool);
    let repo_manager = RepoManager::new(c3p0.clone());

    let cms_config = CmsConfig::build();

    let mut cms_module = CmsModule::new(repo_manager, cms_config);
    cms_module.start().unwrap();

    ((cms_module), node)
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}
