use crate::config::AuthConfig;
use crate::dto::create_user_dto::CreateLoginDto;
use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenModel, TokenType};
use crate::repository::{AuthAccountRepository, AuthRepositoryManager};
use crate::service::password_codec::PasswordCodecService;
use crate::service::token::TokenService;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::auth::Auth;
use lightspeed_core::service::validator::Validator;
use lightspeed_core::utils::current_epoch_seconds;
use lightspeed_email::model::email::EmailMessage;
use lightspeed_email::service::email::EmailService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthAccountService<RepoManager: AuthRepositoryManager> {
    c3p0: RepoManager::C3P0,
    auth_config: AuthConfig,
    auth_repo: RepoManager::AUTH_ACCOUNT_REPO,
    password_service: PasswordCodecService,
    token_service: TokenService<RepoManager::TOKEN_REPO>,
    email_service: Arc<Box<EmailService>>,
}

impl<RepoManager: AuthRepositoryManager> AuthAccountService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        auth_config: AuthConfig,
        token_service: TokenService<RepoManager::TOKEN_REPO>,
        password_service: PasswordCodecService,
        auth_repo: RepoManager::AUTH_ACCOUNT_REPO,
        email_service: Arc<Box<EmailService>>,
    ) -> Self {
        AuthAccountService {
            c3p0,
            auth_config,
            auth_repo,
            password_service,
            token_service,
            email_service,
        }
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Auth, LightSpeedError> {
        let model = self
            .auth_repo
            .fetch_by_username(&self.c3p0.connection()?, username)?
            .filter(|model| match model.data.status {
                AuthAccountStatus::Active => true,
                _ => false,
            });

        if let Some(user) = model {
            if self
                .password_service
                .verify_match(password, &user.data.password)?
            {
                return Ok(Auth {
                    username: user.data.username,
                    id: user.id,
                    roles: user.data.roles,
                });
            }
        };

        Err(LightSpeedError::BadRequest {
            message: "".to_string(),
        })
    }

    pub fn create_user(
        &self,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        let hashed_password = self
            .password_service
            .hash_password(&create_login_dto.password)?;

        let result = self.c3p0.transaction(move |conn| {
            let existing_user = self
                .auth_repo
                .fetch_by_username(conn, &create_login_dto.username)?;
            let existing_email = self
                .auth_repo
                .fetch_by_email(conn, &create_login_dto.email)?;
            Validator::validate((&create_login_dto, |error_details: &mut ErrorDetails| {
                if existing_user.is_some() {
                    error_details.add_detail("username", "NOT_UNIQUE");
                }
                if existing_email.is_some() {
                    error_details.add_detail("email", "NOT_UNIQUE");
                }
                Ok(())
            }))?;

            let auth_account_model = self.auth_repo.save(
                conn,
                NewModel::new(AuthAccountData {
                    username: create_login_dto.username,
                    email: create_login_dto.email,
                    password: hashed_password,
                    roles: vec![],
                    created_date_epoch_seconds: current_epoch_seconds(),
                    status: AuthAccountStatus::PendingActivation,
                }),
            )?;

            let token = self.send_activation_email(
                conn,
                &auth_account_model.data.username,
                &auth_account_model.data.email,
            )?;
            Ok((auth_account_model, token))
        })?;

        Ok(result)
    }

    fn send_activation_email(
        &self,
        conn: &RepoManager::CONN,
        username: &str,
        email: &str,
    ) -> Result<TokenModel, LightSpeedError> {
        let token = self.token_service.generate_and_save_token(
            conn,
            username,
            TokenType::AccountActivation,
        )?;
        let token_public_url = self.token_service.generate_public_token_url(&token);

        let mut email_message = EmailMessage::new();
        email_message.to.push(email.to_owned());
        email_message.subject = Some("Activation link".to_owned());
        email_message.html = Some(format!("Activation link: {}", token_public_url));
        self.email_service.send(email_message)?;

        Ok(token)
    }
}
