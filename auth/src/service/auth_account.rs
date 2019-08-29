use crate::config::AuthConfig;
use crate::dto::change_password_dto::ChangePasswordDto;
use crate::dto::create_user_dto::CreateLoginDto;
use crate::dto::reset_password_dto::ResetPasswordDto;
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
use log::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthAccountService<RepoManager: AuthRepositoryManager> {
    c3p0: RepoManager::C3P0,
    auth_config: AuthConfig,
    auth_repo: RepoManager::AUTH_ACCOUNT_REPO,
    password_service: PasswordCodecService,
    token_service: TokenService<RepoManager::TOKEN_REPO>,
    email_service: Arc<Box<dyn EmailService>>,
}

impl<RepoManager: AuthRepositoryManager> AuthAccountService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        auth_config: AuthConfig,
        token_service: TokenService<RepoManager::TOKEN_REPO>,
        password_service: PasswordCodecService,
        auth_repo: RepoManager::AUTH_ACCOUNT_REPO,
        email_service: Arc<Box<dyn EmailService>>,
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
            .fetch_by_username_optional(&self.c3p0.connection()?, username)?
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

        self.c3p0.transaction(move |conn| {
            let existing_user = self
                .auth_repo
                .fetch_by_username_optional(conn, &create_login_dto.username)?;
            let existing_email = self
                .auth_repo
                .fetch_by_email_optional(conn, &create_login_dto.email)?;
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
        })
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

    pub fn send_new_activation_token_by_email(
        &self,
        previous_activation_token: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let token =
                self.token_service
                    .fetch_by_token(conn, previous_activation_token, false)?;

            Validator::validate(|error_details: &mut ErrorDetails| {
                match &token.data.token_type {
                    TokenType::AccountActivation => {}
                    _ => error_details.add_detail("token_type", "WRONG_TYPE"),
                };
                Ok(())
            })?;

            info!(
                "Send new activation token to user [{}]",
                token.data.username
            );

            let user = self
                .auth_repo
                .fetch_by_username(conn, &token.data.username)?;

            match &user.data.status {
                AuthAccountStatus::PendingActivation => {}
                _ => {
                    return Err(LightSpeedError::BadRequest {
                        message: format!(
                            "User [{}] not in status PendingActivation",
                            token.data.username
                        ),
                    })
                }
            };

            self.token_service.delete(conn, token)?;

            let token = self.send_activation_email(conn, &user.data.username, &user.data.email)?;
            Ok((user, token))
        })
    }

    pub fn activate_user(
        &self,
        activation_token: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let token = self
                .token_service
                .fetch_by_token(conn, activation_token, true)?;

            Validator::validate(|error_details: &mut ErrorDetails| {
                match &token.data.token_type {
                    TokenType::AccountActivation => {}
                    _ => error_details.add_detail("token_type", "WRONG_TYPE"),
                };
                Ok(())
            })?;

            info!("Activate user [{}]", token.data.username);

            let mut user = self
                .auth_repo
                .fetch_by_username(conn, &token.data.username)?;

            match &user.data.status {
                AuthAccountStatus::PendingActivation => {}
                _ => {
                    return Err(LightSpeedError::BadRequest {
                        message: format!(
                            "User [{}] not in status PendingActivation",
                            token.data.username
                        ),
                    })
                }
            };

            self.token_service.delete(conn, token)?;

            user.data.status = AuthAccountStatus::Active;
            user = self.auth_repo.update(conn, user)?;
            Ok(user)
        })
    }

    pub fn send_reset_password_email(
        &self,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        info!("Send reset password email to [{}]", username);

        self.c3p0.transaction(move |conn| {
            let user = self.auth_repo.fetch_by_username(conn, &username)?;

            match &user.data.status {
                AuthAccountStatus::Active => {}
                _ => {
                    return Err(LightSpeedError::BadRequest {
                        message: format!("User [{}] not in status Active", username),
                    })
                }
            };

            let token = self.token_service.generate_and_save_token(
                conn,
                username,
                TokenType::ResetPassword,
            )?;

            let token_public_url = self.token_service.generate_public_token_url(&token);

            let mut email_message = EmailMessage::new();
            email_message.to.push(user.data.email.to_owned());
            email_message.subject = Some("Reset password link".to_owned());
            email_message.html = Some(format!("Reset password link: {}", token_public_url));
            self.email_service.send(email_message)?;

            Ok((user, token))
        })
    }

    pub fn reset_password_by_token(
        &self,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        Validator::validate(&reset_password_dto)?;

        let result = self.c3p0.transaction(move |conn| {
            let token =
                self.token_service
                    .fetch_by_token(conn, &reset_password_dto.token, false)?;

            info!("Reset password of user [{}]", token.data.username);

            Validator::validate(|error_details: &mut ErrorDetails| {
                match &token.data.token_type {
                    TokenType::ResetPassword => {}
                    _ => error_details.add_detail("token_type", "WRONG_TYPE"),
                };
                Ok(())
            })?;

            let mut user = self
                .auth_repo
                .fetch_by_username(conn, &token.data.username)?;

            match &user.data.status {
                AuthAccountStatus::Active => {}
                _ => {
                    return Err(LightSpeedError::BadRequest {
                        message: format!("User [{}] not in status Active", token.data.username),
                    })
                }
            };

            self.token_service.delete(conn, token)?;

            user.data.password = self
                .password_service
                .hash_password(&reset_password_dto.password)?;
            user = self.auth_repo.update(conn, user)?;
            Ok(user)
        });
        Ok(result?)
    }

    pub fn change_password(
        &self,
        dto: ChangePasswordDto,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        Validator::validate(&dto)?;
        self.c3p0.transaction(move |conn| {
            let mut user = self.auth_repo.fetch_by_id(conn, dto.user_id)?;
            info!("Change password of user [{}]", user.data.username);

            match &user.data.status {
                AuthAccountStatus::Active => {}
                _ => {
                    return Err(LightSpeedError::BadRequest {
                        message: format!("User [{}] not in status Active", user.data.username),
                    })
                }
            };

            if !self
                .password_service
                .verify_match(&dto.old_password, &user.data.password)?
            {
                return Err(LightSpeedError::BadRequest {
                    message: "Wrong credentials".to_owned(),
                });
            }

            user.data.password = self.password_service.hash_password(&dto.new_password)?;

            user = self.auth_repo.update(conn, user)?;
            Ok(user)
        })
    }

    pub fn fetch_by_user_id(
        &self,
        conn: &RepoManager::CONN,
        user_id: i64,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.auth_repo.fetch_by_id(conn, user_id)
    }

    pub fn fetch_by_username(
        &self,
        conn: &RepoManager::CONN,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.auth_repo.fetch_by_username(conn, username)
    }
}
