use crate::config::AuthConfig;
use crate::repository::AuthRepositoryManager;
use crate::service::auth_account::LsAuthAccountService;
use crate::service::password_codec::LsPasswordCodecService;
use lightspeed_core::error::{ErrorDetail, LsError, RootErrorDetails};
use lightspeed_validator::error::{ValidationError, ValidatorError};
use log::*;
use std::sync::Arc;

pub(crate) fn into_ls_error(e: ValidatorError) -> LsError {
    match e {
        ValidatorError::ValidationFailed { details } => LsError::ValidationError {
            details: RootErrorDetails {
                message: None,
                details: details
                    .details
                    .into_iter()
                    .map(|(k, errs)| (k, errs.into_iter().map(validation_error_to_detail).collect()))
                    .collect(),
            },
        },
        ValidatorError::Error(e) => LsError::InternalServerError { message: e.to_string() },
    }
}

fn validation_error_to_detail(e: ValidationError) -> ErrorDetail {
    match e {
        ValidationError::MustBeTrue => ErrorDetail::new("MUST_BE_TRUE", vec![]),
        ValidationError::MustBeFalse => ErrorDetail::new("MUST_BE_FALSE", vec![]),
        ValidationError::MustContain { needle } => ErrorDetail::new("MUST_CONTAIN", vec![needle]),
        ValidationError::NotValidEmail => ErrorDetail::new("NOT_VALID_EMAIL", vec![]),
        ValidationError::NotValidIp => ErrorDetail::new("NOT_VALID_IP", vec![]),
        ValidationError::NotEquals { a, b } => ErrorDetail::new("NOT_EQUALS", vec![a, b]),
        ValidationError::MustBeLessOrEqual { max } => ErrorDetail::new("MUST_BE_LESS_OR_EQUAL", vec![max]),
        ValidationError::MustBeLess { max } => ErrorDetail::new("MUST_BE_LESS", vec![max]),
        ValidationError::MustBeGreaterOrEqual { min } => ErrorDetail::new("MUST_BE_GREATER_OR_EQUAL", vec![min]),
        ValidationError::MustBeGreater { min } => ErrorDetail::new("MUST_BE_GREATER", vec![min]),
        ValidationError::NotValidUrl => ErrorDetail::new("NOT_VALID_URL", vec![]),
        ValidationError::WrongOwner => ErrorDetail::new("WRONG_OWNER", vec![]),
        ValidationError::WrongId => ErrorDetail::new("WRONG_ID", vec![]),
        ValidationError::WrongVersion => ErrorDetail::new("WRONG_VERSION", vec![]),
        ValidationError::NotUnique => ErrorDetail::new("NOT_UNIQUE", vec![]),
        ValidationError::ValueRequired => ErrorDetail::new("VALUE_REQUIRED", vec![]),
        ValidationError::UnknownField => ErrorDetail::new("UNKNOWN_FIELD", vec![]),
        ValidationError::Custom { code, params } => ErrorDetail::new(code, params),
    }
}

pub mod config;
pub mod dto;
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
    pub fn new(repo_manager: RepoManager, auth_config: AuthConfig) -> Self {
        println!("Creating LsAuthModule");
        info!("Creating LsAuthModule");

        let password_codec = Arc::new(LsPasswordCodecService::new(
            auth_config.argon2_memory_kib,
            auth_config.argon2_iterations,
            auth_config.argon2_parallelism,
        ));

        let token_service =
            Arc::new(service::token::LsTokenService::new(auth_config.clone(), repo_manager.token_repo()));

        let auth_account_service = Arc::new(LsAuthAccountService::new(
            repo_manager.c3p0().clone(),
            auth_config.clone(),
            token_service.clone(),
            password_codec.clone(),
            repo_manager.auth_account_repo(),
        ));

        LsAuthModule { auth_config, repo_manager, password_codec, auth_account_service, token_service }
    }
}

impl<RepoManager: AuthRepositoryManager> lightspeed_core::module::LsModule for LsAuthModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsAuthModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
