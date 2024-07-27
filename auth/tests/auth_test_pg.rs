use std::sync::OnceLock;

use c3p0::sqlx::sqlx::postgres::*;
use c3p0::sqlx::*;
use maybe_single::tokio::*;

use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::pg::PgAuthRepositoryManager;
use lightspeed_auth::LsAuthModule;
use lightspeed_core::module::LsModule;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::runners::AsyncRunner;
use testcontainers::testcontainers::ContainerAsync;

mod tests;

pub type Id = u64;
pub type RepoManager = PgAuthRepositoryManager<Id>;

pub type MaybeType = (LsAuthModule<Id, RepoManager>, ContainerAsync<Postgres>);

async fn init() -> MaybeType {
    let node = Postgres::default().start().await.unwrap();

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(5432).await.unwrap());

    let pool = PgPool::connect_with(options).await.unwrap();

    let c3p0 = SqlxPgC3p0Pool::new(pool);

    let repo_manager = RepoManager::new(c3p0.clone());

    let auth_config = AuthConfig { bcrypt_password_hash_cost: 4, ..Default::default() };

    let mut auth_module = LsAuthModule::new(repo_manager, auth_config);
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeSingleAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init()))).data(serial).await
}

pub fn test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Should create a tokio runtime")
    })
    .block_on(f)
}
