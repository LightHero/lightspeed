use crate::config::AuthConfig;
use crate::dto::change_password_dto::ChangePasswordDto;
use crate::dto::create_login_dto::CreateLoginDto;
use crate::dto::reset_password_dto::ResetPasswordDto;
use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenModel, TokenType};
use crate::repository::{AuthAccountRepository, AuthRepositoryManager};
use crate::service::password_codec::PasswordCodecService;
use crate::service::token::TokenService;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::auth::Auth;
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use lightspeed_core::utils::current_epoch_seconds;
use log::*;

pub const WRONG_TYPE: &str = "WRONG_TYPE";

#[derive(Clone)]
pub struct AuthAccountService<RepoManager: AuthRepositoryManager> {
    c3p0: RepoManager::C3P0,
    auth_config: AuthConfig,
    auth_repo: RepoManager::AuthAccountRepo,
    password_service: PasswordCodecService,
    token_service: TokenService<RepoManager>,
}

impl<RepoManager: AuthRepositoryManager> AuthAccountService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        auth_config: AuthConfig,
        token_service: TokenService<RepoManager>,
        password_service: PasswordCodecService,
        auth_repo: RepoManager::AuthAccountRepo,
    ) -> Self {
        AuthAccountService {
            c3p0,
            auth_config,
            auth_repo,
            password_service,
            token_service,
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Auth, LightSpeedError> {
        debug!("login attempt with username [{}]", username);
        let model = self
            .c3p0
            .transaction(|mut conn| async move {
                self.auth_repo
                    .fetch_by_username_optional(&mut conn, username)
                    .await
            })
            .await?
            .filter(|model| match model.data.status {
                AuthAccountStatus::ACTIVE => true,
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

    pub async fn create_user(
        &self,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.create_user_with_conn(&mut conn, create_login_dto)
                    .await
            })
            .await
    }

    pub async fn create_user_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        info!(
            "Create login attempt with username [{:?}] and email [{}]",
            create_login_dto.username, create_login_dto.email
        );
        let hashed_password = self
            .password_service
            .hash_password(&create_login_dto.password)?;

        let username = match &create_login_dto.username {
            Some(username) => {
                if !username.is_empty() {
                    username
                } else {
                    &create_login_dto.email
                }
            }
            None => &create_login_dto.email,
        }
        .clone();

        let existing_user = self
            .auth_repo
            .fetch_by_username_optional(conn, &username)
            .await?;
        let existing_email = self
            .auth_repo
            .fetch_by_email_optional(conn, &create_login_dto.email)
            .await?;
        Validator::validate(&(&create_login_dto, &|error_details: &mut ErrorDetails| {
            if existing_user.is_some() {
                error_details.add_detail("username", ERR_NOT_UNIQUE);
            }
            if existing_email.is_some() {
                error_details.add_detail("email", ERR_NOT_UNIQUE);
            }
            Ok(())
        }))?;

        let auth_account_model = self
            .auth_repo
            .save(
                conn,
                NewModel::new(AuthAccountData {
                    username,
                    email: create_login_dto.email,
                    password: hashed_password,
                    roles: self.auth_config.default_roles_on_account_creation.clone(),
                    created_date_epoch_seconds: current_epoch_seconds(),
                    status: AuthAccountStatus::PENDING_ACTIVATION,
                }),
            )
            .await?;

        let token = self
            .generate_activation_token(conn, &auth_account_model.data.username)
            .await?;
        Ok((auth_account_model, token))
    }

    async fn generate_activation_token(
        &self,
        conn: &mut RepoManager::Conn,
        username: &str,
    ) -> Result<TokenModel, LightSpeedError> {
        debug!("Generate activation token for username [{}]", username);
        self.token_service
            .generate_and_save_token(conn, username, TokenType::ACCOUNT_ACTIVATION)
            .await
    }

    pub async fn generate_new_activation_token(
        &self,
        previous_activation_token: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.generate_new_activation_token_with_conn(&mut conn, previous_activation_token)
                    .await
            })
            .await
    }

    pub async fn generate_new_activation_token_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        previous_activation_token: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        debug!(
            "Generate new activation token from previous token [{}]",
            previous_activation_token
        );
        let token = self
            .token_service
            .fetch_by_token(conn, previous_activation_token, false)
            .await?;

        Validator::validate(&|error_details: &mut ErrorDetails| {
            match &token.data.token_type {
                TokenType::ACCOUNT_ACTIVATION => {}
                _ => error_details.add_detail("token_type", WRONG_TYPE),
            };
            Ok(())
        })?;

        info!(
            "Send new activation token to user [{}]",
            token.data.username
        );

        let user = self
            .auth_repo
            .fetch_by_username(conn, &token.data.username)
            .await?;

        match &user.data.status {
            AuthAccountStatus::PENDING_ACTIVATION => {}
            _ => {
                return Err(LightSpeedError::BadRequest {
                    message: format!(
                        "User [{}] not in status PendingActivation",
                        token.data.username
                    ),
                })
            }
        };

        self.token_service.delete(conn, token).await?;

        let token = self
            .generate_activation_token(conn, &user.data.username)
            .await?;
        Ok((user, token))
    }

    pub async fn activate_user(
        &self,
        activation_token: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        debug!("Activate user called with token [{}]", activation_token);
        self.c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                let token = self
                    .token_service
                    .fetch_by_token(conn, activation_token, true)
                    .await?;

                Validator::validate(&|error_details: &mut ErrorDetails| {
                    match &token.data.token_type {
                        TokenType::ACCOUNT_ACTIVATION => {}
                        _ => error_details.add_detail("token_type", WRONG_TYPE),
                    };
                    Ok(())
                })?;

                info!("Activate user [{}]", token.data.username);

                let mut user = self
                    .auth_repo
                    .fetch_by_username(conn, &token.data.username)
                    .await?;

                match &user.data.status {
                    AuthAccountStatus::PENDING_ACTIVATION => {}
                    _ => {
                        return Err(LightSpeedError::BadRequest {
                            message: format!(
                                "User [{}] not in status PendingActivation",
                                token.data.username
                            ),
                        })
                    }
                };

                self.token_service.delete(conn, token).await?;

                user.data.status = AuthAccountStatus::ACTIVE;
                user = self.auth_repo.update(conn, user).await?;
                Ok(user)
            })
            .await
    }

    pub async fn generate_reset_password_token(
        &self,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.generate_reset_password_token_with_conn(&mut conn, username)
                    .await
            })
            .await
    }

    pub async fn generate_reset_password_token_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LightSpeedError> {
        info!("Generate reset password token for username [{}]", username);

        let user = self.auth_repo.fetch_by_username(conn, &username).await?;

        match &user.data.status {
            AuthAccountStatus::ACTIVE => {}
            _ => {
                return Err(LightSpeedError::BadRequest {
                    message: format!("User [{}] not in status Active", username),
                })
            }
        };

        let token = self
            .token_service
            .generate_and_save_token(conn, username, TokenType::RESET_PASSWORD)
            .await?;

        Ok((user, token))
    }

    pub async fn reset_password_by_token(
        &self,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        debug!(
            "Reset password called with token [{}]",
            reset_password_dto.token
        );
        Validator::validate(&reset_password_dto)?;

        let result = self
            .c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                let token = self
                    .token_service
                    .fetch_by_token(conn, &reset_password_dto.token, false)
                    .await?;

                info!("Reset password of user [{}]", token.data.username);

                Validator::validate(&|error_details: &mut ErrorDetails| {
                    match &token.data.token_type {
                        TokenType::RESET_PASSWORD => {}
                        _ => error_details.add_detail("token_type", WRONG_TYPE),
                    };
                    Ok(())
                })?;

                let mut user = self
                    .auth_repo
                    .fetch_by_username(conn, &token.data.username)
                    .await?;

                match &user.data.status {
                    AuthAccountStatus::ACTIVE => {}
                    _ => {
                        return Err(LightSpeedError::BadRequest {
                            message: format!("User [{}] not in status Active", token.data.username),
                        })
                    }
                };

                self.token_service.delete(conn, token).await?;

                user.data.password = self
                    .password_service
                    .hash_password(&reset_password_dto.password)?;
                user = self.auth_repo.update(conn, user).await?;
                Ok(user)
            })
            .await;
        Ok(result?)
    }

    pub async fn change_password(
        &self,
        dto: ChangePasswordDto,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        info!("Reset password of user_id [{}]", dto.user_id);

        Validator::validate(&dto)?;
        self.c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                let mut user = self.auth_repo.fetch_by_id(conn, dto.user_id).await?;
                info!("Change password of user [{}]", user.data.username);

                match &user.data.status {
                    AuthAccountStatus::ACTIVE => {}
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

                user = self.auth_repo.update(conn, user).await?;
                Ok(user)
            })
            .await
    }

    pub async fn fetch_by_user_id_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        user_id: i64,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        debug!("Fetch user with user_id [{}]", user_id);
        self.auth_repo.fetch_by_id(conn, user_id).await
    }

    pub async fn fetch_by_username(
        &self,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.fetch_by_username_with_conn(&mut conn, username).await
            })
            .await
    }

    pub async fn fetch_by_username_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        debug!("Fetch user with username [{}]", username);
        self.auth_repo.fetch_by_username(conn, username).await
    }

    pub async fn add_roles(
        &self,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.add_roles_with_conn(&mut conn, user_id, roles).await
            })
            .await
    }

    pub async fn add_roles_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LightSpeedError> {
        info!("Add roles [{:?}] to user_id [{}]", roles, user_id);

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            if !account.data.roles.contains(role) {
                account.data.roles.push(role.to_owned())
            }
        }
        Ok(self.auth_repo.update(conn, account).await?)
    }

    pub async fn delete_roles(
        &self,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                self.delete_roles_with_conn(&mut conn, user_id, roles).await
            })
            .await
    }

    pub async fn delete_roles_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LightSpeedError> {
        info!("delete roles [{:?}] to user_id [{}]", roles, user_id);

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            account.data.roles.retain(|r| r != role);
        }
        Ok(self.auth_repo.update(conn, account).await?)
    }
}
