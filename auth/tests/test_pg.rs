use c3p0::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use testcontainers::*;

use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::repository::pg::PgAuthRepositoryManager;
use lightspeed_auth::AuthModule;
use lightspeed_core::config::UIConfig;
use lightspeed_core::module::Module;
use lightspeed_email::config::EmailConfig;
use lightspeed_email::service::email::EmailServiceType;
use lightspeed_email::EmailModule;

mod tests;

pub type RepoManager = PgAuthRepositoryManager;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        (AuthModule<RepoManager>, EmailModule),
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    (AuthModule<RepoManager>, EmailModule),
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

    let mut email_config = EmailConfig::build();
    email_config.service_type = EmailServiceType::InMemory;
    let email_module = EmailModule::new(email_config, c3p0.clone()).unwrap();

    let mut auth_config = AuthConfig::build();
    auth_config.bcrypt_password_hash_cost = 4;

    let ui_config = UIConfig::build();
    let mut auth_module = AuthModule::new(repo_manager, auth_config, ui_config, &email_module);
    auth_module.start().unwrap();

    ((auth_module, email_module), node)
}

pub fn test(
    callback: fn(&AuthModule<RepoManager>, &EmailModule) -> Result<(), Box<dyn std::error::Error>>,
) {
    SINGLETON.get(|((auth_module, email_module), _)| {
        email_module.email_service.clear_emails().unwrap();
        callback(&auth_module, &email_module).unwrap();
    });
}
