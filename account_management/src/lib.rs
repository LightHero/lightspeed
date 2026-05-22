// No `unsafe` in this crate.
#![forbid(unsafe_code)]
// `.unwrap()` and `.expect()` are banned in production code.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

use crate::config::AMConfig;
use crate::repository::AMRepositoryManager;
use crate::service::account::LsAMAccountService;
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
pub struct LsAMModule<RepoManager: AMRepositoryManager> {
    pub auth_config: AMConfig,

    pub repo_manager: RepoManager,

    pub password_codec: Arc<service::password_codec::LsPasswordCodecService>,
    pub auth_account_service: Arc<service::account::LsAMAccountService<RepoManager>>,
    pub token_service: Arc<service::token::LsTokenService<RepoManager>>,
}

impl<RepoManager: AMRepositoryManager> LsAMModule<RepoManager> {
    /// Builds the module.
    pub fn new(repo_manager: RepoManager, auth_config: AMConfig) -> Result<Self, error::LsAccountManagementError> {
        println!("Creating LsAMModule");
        info!("Creating LsAMModule");

        let password_codec = Arc::new(LsPasswordCodecService::new(
            auth_config.argon2_memory_kib,
            auth_config.argon2_iterations,
            auth_config.argon2_parallelism,
        )?);

        let token_service =
            Arc::new(service::token::LsTokenService::new(auth_config.clone(), repo_manager.token_repo()));

        let auth_account_service = Arc::new(LsAMAccountService::new(
            repo_manager.c3p0().clone(),
            auth_config.clone(),
            token_service.clone(),
            password_codec.clone(),
            repo_manager.account_repo(),
        ));

        Ok(LsAMModule { auth_config, repo_manager, password_codec, auth_account_service, token_service })
    }
}

impl<RepoManager: AMRepositoryManager> lightspeed_core::module::LsModule for LsAMModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsAMModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
