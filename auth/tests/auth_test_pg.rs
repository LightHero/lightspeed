use c3p0::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0::pg::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use testcontainers::*;

use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::pg::PgAuthRepositoryManager;
use lightspeed_auth::AuthModule;
use lightspeed_core::module::Module;

mod tests;

pub type RepoManager = PgAuthRepositoryManager;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        AuthModule<RepoManager>,
        Container<'static, clients::Cli, images::generic::GenericImage>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    AuthModule<RepoManager>,
    Container<'static, clients::Cli, images::generic::GenericImage>,
) {
    let node = DOCKER.run(images::generic::GenericImage::new("postgres:11-alpine")
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres"));

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();
    let c3p0 = PgC3p0Pool::new(pool);
    let repo_manager = RepoManager::new(c3p0.clone());

    let mut auth_config = AuthConfig::build();
    auth_config.bcrypt_password_hash_cost = 4;

    let mut auth_module = AuthModule::new(repo_manager, auth_config);
    auth_module.start().unwrap();

    (auth_module, node)
}

pub fn test(
    no_parallel: bool,
    callback: fn(&AuthModule<RepoManager>) -> Result<(), Box<dyn std::error::Error>>,
) {
    SINGLETON.get(no_parallel, |(auth_module, _)| {
        callback(&auth_module).unwrap();
    });
}
