use c3p0::pg_async::deadpool;
use c3p0::pg_async::driver::NoTls;
use c3p0::pg_async::*;
use lazy_static::lazy_static;
use maybe_single::nio::*;
use testcontainers::*;

use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::pg::PgAuthRepositoryManager;
use lightspeed_auth::AuthModule;
use lightspeed_core::module::Module;
use tokio::time::Duration;

mod tests;

pub type RepoManager = PgAuthRepositoryManager;

pub type MaybeType = (
    AuthModule<RepoManager>,
    Container<'static, clients::Cli, images::postgres::Postgres>,
);

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingleAsync<MaybeType> = MaybeSingleAsync::new(|| init().boxed());
}

async fn init() -> MaybeType {
    let node = DOCKER.run(images::postgres::Postgres::default());

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

    let c3p0 = PgC3p0PoolAsync::new(config.create_pool(NoTls).unwrap());

    let repo_manager = RepoManager::new(c3p0.clone());

    let mut auth_config = AuthConfig::build();
    auth_config.bcrypt_password_hash_cost = 4;

    let mut auth_module = AuthModule::new(repo_manager, auth_config);
    auth_module.start().await.unwrap();

    (auth_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial).await
}

fn rt() -> &'static tokio::runtime::Runtime {
    lazy_static! {
        static ref RT: tokio::runtime::Runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Should create a tokio runtime");
    }
    &RT
}