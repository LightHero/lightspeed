use crate::config::AuthConfig;
use crate::service::password_codec::PasswordCodecService;
use c3p0::*;
use lightspeed_core::{config::UIConfig, error::LightSpeedError};
use log::*;
use crate::service::auth_account::AuthAccountService;
use crate::repository::db::AuthDbRepository;

pub mod config;
pub mod model;
pub mod repository;
pub mod service;

pub type PoolManager = PgPoolManager;

#[derive(Clone)]
pub struct AuthModule {
    pub ui_config: UIConfig,
    pub auth_config: AuthConfig,

    pub c3p0: C3p0Pool<PoolManager>,

    db_repo: AuthDbRepository,
    pub auth_repo: repository::auth_account::AuthAccountRepository,
    pub token_repo: repository::token::TokenRepository,

    pub password_codec: service::password_codec::PasswordCodecService,
    pub auth_account_service: service::auth_account::AuthAccountService,
    pub token_service: service::token::TokenService,
}

impl AuthModule {
    pub fn new(auth_config: AuthConfig, ui_config: UIConfig, c3p0: C3p0Pool<PoolManager>) -> Self {
        println!("Creating AuthModule");
        info!("Creating AuthModule");

        let db_repo = repository::db::AuthDbRepository::new(c3p0.clone());
        let auth_repo = repository::auth_account::AuthAccountRepository::new();
        let token_repo = repository::token::TokenRepository::new();

        let password_codec = PasswordCodecService::new(auth_config.bcrypt_password_hash_cost);

        let token_service = service::token::TokenService::new(
            auth_config.clone(),
            ui_config.clone(),
            token_repo.clone(),
        );

        let auth_account_service = AuthAccountService::new(
            c3p0.clone(),
            auth_config.clone(),
            token_service.clone(),
            password_codec.clone(),
            auth_repo.clone(),
        );

        AuthModule {
            ui_config,
            auth_config,

            c3p0,

            db_repo,
            auth_repo,
            token_repo,

            password_codec,
            auth_account_service,
            token_service,
        }
    }
}

impl lightspeed_core::module::Module for AuthModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting AuthModule");
        self.db_repo.start()?;
        Ok(())
    }
}

#[cfg(test)]
pub mod test_root {

    use lazy_static::lazy_static;
    use ls_logger::config::LoggerConfig;
    use ls_logger::setup_logger;
    use std::sync::Mutex;

    lazy_static! {
        static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
    }

    pub fn init_context() {
        let mut init = INITIALIZED.lock().unwrap();
        if !*init {
            println!("Initialize context");
            start_logger();
            *init = true;
        }
    }

    fn start_logger() {
        println!("Init logger");

        let conf = LoggerConfig {
            level: String::from("trace"),
            stdout_output: true,
            stderr_output: false,
            file_output_path: None,
        };
        setup_logger(&conf).unwrap();
    }

}
