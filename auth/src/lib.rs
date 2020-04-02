use crate::config::AuthConfig;
use crate::repository::AuthRepositoryManager;
use crate::service::auth_account::AuthAccountService;
use crate::service::password_codec::PasswordCodecService;
use lightspeed_core::error::LightSpeedError;
use log::*;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct AuthModule<RepoManager: AuthRepositoryManager> {
    pub auth_config: AuthConfig,

    pub repo_manager: RepoManager,

    pub password_codec: service::password_codec::PasswordCodecService,
    pub auth_account_service: service::auth_account::AuthAccountService<RepoManager>,
    pub token_service: service::token::TokenService<RepoManager>,
}

impl<RepoManager: AuthRepositoryManager> AuthModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, auth_config: AuthConfig) -> Self {
        println!("Creating AuthModule");
        info!("Creating AuthModule");

        let password_codec = PasswordCodecService::new(auth_config.bcrypt_password_hash_cost);

        let token_service =
            service::token::TokenService::new(auth_config.clone(), repo_manager.token_repo());

        let auth_account_service = AuthAccountService::new(
            repo_manager.c3p0().clone(),
            auth_config.clone(),
            token_service.clone(),
            password_codec.clone(),
            repo_manager.auth_account_repo(),
        );

        AuthModule {
            auth_config,
            repo_manager,
            password_codec,
            auth_account_service,
            token_service,
        }
    }
}

#[async_trait::async_trait]
impl<RepoManager: AuthRepositoryManager> lightspeed_core::module::Module
    for AuthModule<RepoManager>
{
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting AuthModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}

#[cfg(test)]
pub mod test_root {

    use lazy_static::lazy_static;
    use lightspeed_logger::config::LoggerConfig;
    use lightspeed_logger::setup_logger;
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
        if let Err(err) = setup_logger(&conf) {
            println!("Warning: {}", err)
        };
    }
}
