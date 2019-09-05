use c3p0::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0::pg::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use testcontainers::*;

use lightspeed_cms::config::CmsConfig;
use lightspeed_cms::repository::pg::PgCmsRepositoryManager;
use lightspeed_cms::CmsModule;
use lightspeed_core::config::UIConfig;
use lightspeed_core::module::Module;

mod tests;

pub type RepoManager = PgCmsRepositoryManager;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        (CmsModule<RepoManager>),
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    (CmsModule<RepoManager>),
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
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
    let c3p0 = C3p0PoolPg::new(pool);
    let repo_manager = RepoManager::new(c3p0.clone());

    let cms_config = CmsConfig::build();

    let ui_config = UIConfig::build();
    let mut cms_module = CmsModule::new(repo_manager, cms_config, ui_config);
    cms_module.start().unwrap();

    ((cms_module), node)
}

pub fn test(callback: fn(&CmsModule<RepoManager>) -> Result<(), Box<dyn std::error::Error>>) {
    SINGLETON.get(|(cms_module, _)| {
        callback(&cms_module).unwrap();
    });
}
