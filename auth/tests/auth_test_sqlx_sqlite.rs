#![cfg(feature = "sqlx_sqlite")]

use std::sync::OnceLock;

use maybe_once::tokio::*;

use lightspeed_auth::LsAuthModule;
use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::sqlx_sqlite::SqliteAuthRepositoryManager;
use lightspeed_core::module::LsModule;
use lightspeed_test_utils::sqlite::new_sqlite_db;

mod tests;

pub type RepoManager = SqliteAuthRepositoryManager;

pub type MaybeType = (LsAuthModule<RepoManager>, ());

async fn init() -> MaybeType {
    let c3p0 = new_sqlite_db().await;

    let repo_manager = RepoManager::new(c3p0.clone());

    let auth_config = AuthConfig { bcrypt_password_hash_cost: 4, ..Default::default() };

    let mut auth_module = LsAuthModule::new(repo_manager, auth_config);
    {
        auth_module.start().await.unwrap();
    }

    (auth_module, ())
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init()))).data(serial).await
}
