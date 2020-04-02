use c3p0::pg_async::bb8::{Pool, PostgresConnectionManager};
use c3p0::pg_async::driver::tls::NoTls;
use c3p0::pg_async::*;
use lazy_static::lazy_static;
use maybe_single::nio::*;
use testcontainers::*;

use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::pg::PgAuthRepositoryManager;
use lightspeed_auth::AuthModule;
use lightspeed_core::module::Module;

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

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        )
            .parse()
            .unwrap(),
        NoTls,
    );

    let pool = Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .await
        .unwrap();
    let c3p0 = PgC3p0PoolAsync::new(pool);

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
