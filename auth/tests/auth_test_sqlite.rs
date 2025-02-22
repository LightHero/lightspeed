#![cfg(feature = "sqlite")]

use std::sync::OnceLock;

use c3p0::sqlx::sqlx::sqlite::*;
use c3p0::sqlx::*;
use maybe_single::tokio::*;

use lightspeed_auth::LsAuthModule;
use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::sqlite::SqliteAuthRepositoryManager;
use lightspeed_core::module::LsModule;

mod tests;

pub type RepoManager = SqliteAuthRepositoryManager;

pub type MaybeType = (LsAuthModule<RepoManager>, ());

async fn init() -> MaybeType {
    let options = SqliteConnectOptions::new().in_memory(true);

    let pool: c3p0::sqlx::sqlx::Pool<Sqlite> = c3p0::sqlx::sqlx::pool::PoolOptions::new()
        .max_lifetime(None)
        .idle_timeout(None)
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    let c3p0 = SqlxSqliteC3p0Pool::new(pool);

    let repo_manager = RepoManager::new(c3p0.clone());

    let auth_config = AuthConfig { bcrypt_password_hash_cost: 4, ..Default::default() };

    let mut auth_module = LsAuthModule::new(repo_manager, auth_config);
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, ())
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
