// No `unsafe` in this crate.
#![forbid(unsafe_code)]
// `.unwrap()` and `.expect()` are banned in production code.
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used)
)]

use crate::config::AuthConfig;
use crate::repository::AuthRepositoryManager;
use crate::service::auth_account::LsAuthAccountService;
use crate::service::password_codec::LsPasswordCodecService;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod dto;
pub mod error;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct LsAuthModule<RepoManager: AuthRepositoryManager> {
    pub auth_config: AuthConfig,

    pub repo_manager: RepoManager,

    pub password_codec: Arc<service::password_codec::LsPasswordCodecService>,
    pub auth_account_service: Arc<service::auth_account::LsAuthAccountService<RepoManager>>,
    pub token_service: Arc<service::token::LsTokenService<RepoManager>>,
}

impl<RepoManager: AuthRepositoryManager> LsAuthModule<RepoManager> {
    /// Builds the module. Returns
    /// [`LsAccountManagerError::PasswordEncryptionError`] if the
    /// [`AuthConfig`]'s Argon2 parameters are invalid (the only fallible
    /// step inside; everything else is infallible construction).
    pub fn new(
        repo_manager: RepoManager,
        auth_config: AuthConfig,
    ) -> Result<Self, error::LsAccountManagerError> {
        println!("Creating LsAuthModule");
        info!("Creating LsAuthModule");

        let password_codec = Arc::new(LsPasswordCodecService::new(
            auth_config.argon2_memory_kib,
            auth_config.argon2_iterations,
            auth_config.argon2_parallelism,
        )?);

        let token_service =
            Arc::new(service::token::LsTokenService::new(auth_config.clone(), repo_manager.token_repo()));

        let auth_account_service = Arc::new(LsAuthAccountService::new(
            repo_manager.c3p0().clone(),
            auth_config.clone(),
            token_service.clone(),
            password_codec.clone(),
            repo_manager.auth_account_repo(),
        ));

        Ok(LsAuthModule { auth_config, repo_manager, password_codec, auth_account_service, token_service })
    }
}

impl<RepoManager: AuthRepositoryManager> lightspeed_core::module::LsModule for LsAuthModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsAuthModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
