use crate::config::AuthConfig;
use crate::dto::change_password_dto::ChangePasswordDto;
use crate::dto::create_login_dto::CreateLoginDto;
use crate::dto::reset_password_dto::ResetPasswordDto;
use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenModel, TokenType};
use crate::repository::{AuthAccountRepository, AuthRepositoryManager};
use crate::service::password_codec::LsPasswordCodecService;
use crate::service::token::LsTokenService;
use c3p0::*;
use lightspeed_core::error::*;
use lightspeed_core::service::auth::Auth;
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use std::sync::Arc;

pub const WRONG_TYPE: &str = "WRONG_TYPE";

#[derive(Clone)]
pub struct LsAuthAccountService<RepoManager: AuthRepositoryManager> {
    c3p0: RepoManager::C3P0,
    auth_config: AuthConfig,
    auth_repo: RepoManager::AuthAccountRepo,
    password_service: Arc<LsPasswordCodecService>,
    token_service: Arc<LsTokenService<RepoManager>>,
}

impl<RepoManager: AuthRepositoryManager> LsAuthAccountService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        auth_config: AuthConfig,
        token_service: Arc<LsTokenService<RepoManager>>,
        password_service: Arc<LsPasswordCodecService>,
        auth_repo: RepoManager::AuthAccountRepo,
    ) -> Self {
        LsAuthAccountService { c3p0, auth_config, auth_repo, password_service, token_service }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Auth<u64>, LsError> {
        self.c3p0.transaction(async |conn| { self.login_with_conn(conn, username, password).await }).await
    }

    pub async fn login_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
        password: &str,
    ) -> Result<Auth<u64>, LsError> {
        debug!("login attempt with username [{}]", username);
        let model = self.auth_repo.fetch_by_username_optional(conn, username).await?;

        if let Some(user) = model {
            if self.password_service.verify_match(password, &user.data.password)? {
                match &user.data.status {
                    AuthAccountStatus::Active => {}
                    _ => {
                        return Err(LsError::BadRequest {
                            message: format!("User [{username}] not in status Active"),
                            code: ErrorCodes::INACTIVE_USER,
                        })
                    }
                };

                let creation_ts_seconds = current_epoch_seconds();
                let expiration_ts_seconds =
                    creation_ts_seconds + (self.auth_config.auth_session_max_validity_minutes * 60);

                return Ok(Auth::new(
                    user.id,
                    user.data.username,
                    user.data.roles,
                    creation_ts_seconds,
                    expiration_ts_seconds,
                ));
            }
        };

        Err(LsError::BadRequest { message: "Wrong credentials".to_string(), code: ErrorCodes::WRONG_CREDENTIALS })
    }

    pub async fn create_user(
        &self,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        self.c3p0.transaction(async |conn| { self.create_user_with_conn(conn, create_login_dto).await }).await
    }

    pub async fn create_user_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        info!(
            "Create login attempt with username [{:?}] and email [{}]",
            create_login_dto.username, create_login_dto.email
        );
        let hashed_password = self.password_service.hash_password(&create_login_dto.password)?;

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

        let existing_user = self.auth_repo.fetch_by_username_optional(conn, &username).await?;
        let existing_email = self.auth_repo.fetch_by_email_optional(conn, &create_login_dto.email).await?;
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
                    status: AuthAccountStatus::PendingActivation,
                }),
            )
            .await?;

        let token = self.generate_activation_token_with_conn(conn, &auth_account_model.data.username).await?;
        Ok((auth_account_model, token))
    }

    async fn generate_activation_token_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
    ) -> Result<TokenModel, LsError> {
        debug!("Generate activation token for username [{}]", username);
        self.token_service.generate_and_save_token_with_conn(conn, username, TokenType::AccountActivation).await
    }

    pub async fn generate_new_activation_token_by_username_and_email(
        &self,
        username: &str,
        email: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.generate_new_activation_token_by_username_and_email_with_conn(conn, username, email).await
            })
            .await
    }

    pub async fn generate_new_activation_token_by_username_and_email_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
        email: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        debug!("Generate new activation token for username [{}] and email [{}]", username, email);
        let auth_account = self.fetch_by_username_with_conn(conn, username).await?;

        Validator::validate(&|error_details: &mut ErrorDetails| {
            if auth_account.data.email != email {
                error_details.add_detail("email", "WRONG_EMAIL");
            };
            Ok(())
        })?;

        let previous_activation_token = self
            .token_service
            .fetch_all_by_username_with_conn(conn, username)
            .await?
            .into_iter()
            .filter(|token| token.data.token_type == TokenType::AccountActivation)
            .map(|token| token.data.token)
            .collect::<Vec<String>>()
            .first()
            .cloned()
            .ok_or_else(|| LsError::BadRequest {
                message: format!("Previous activation token not found for user [{username}]"),
                code: ErrorCodes::NOT_PENDING_USER,
            })?;
        self.generate_new_activation_token_by_token_with_conn(conn, &previous_activation_token).await
    }

    pub async fn generate_new_activation_token_by_token(
        &self,
        previous_activation_token: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.generate_new_activation_token_by_token_with_conn(conn, previous_activation_token).await
            })
            .await
    }

    pub async fn generate_new_activation_token_by_token_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        previous_activation_token: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        debug!("Generate new activation token from previous token [{}]", previous_activation_token);
        let token = self.token_service.fetch_by_token_with_conn(conn, previous_activation_token, false).await?;

        Validator::validate(&|error_details: &mut ErrorDetails| {
            match &token.data.token_type {
                TokenType::AccountActivation => {}
                _ => error_details.add_detail("token_type", WRONG_TYPE),
            };
            Ok(())
        })?;

        info!("Send new activation token to user [{}]", token.data.username);

        let user = self.auth_repo.fetch_by_username(conn, &token.data.username).await?;

        match &user.data.status {
            AuthAccountStatus::PendingActivation => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{}] not in status PendingActivation", token.data.username),
                    code: ErrorCodes::NOT_PENDING_USER,
                })
            }
        };

        self.token_service.delete_with_conn(conn, token).await?;

        let token = self.generate_activation_token_with_conn(conn, &user.data.username).await?;
        Ok((user, token))
    }

    pub async fn activate_user(&self, activation_token: &str) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.activate_user_with_conn(conn, activation_token).await }).await
    }

    pub async fn activate_user_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        activation_token: &str,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Activate user called with token [{}]", activation_token);

        let token = self.token_service.fetch_by_token_with_conn(conn, activation_token, true).await?;

        Validator::validate(&|error_details: &mut ErrorDetails| {
            match &token.data.token_type {
                TokenType::AccountActivation => {}
                _ => error_details.add_detail("token_type", WRONG_TYPE),
            };
            Ok(())
        })?;

        info!("Activate user [{}]", token.data.username);

        let mut user = self.auth_repo.fetch_by_username(conn, &token.data.username).await?;

        match &user.data.status {
            AuthAccountStatus::PendingActivation => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{}] not in status PendingActivation", token.data.username),
                    code: ErrorCodes::NOT_PENDING_USER,
                })
            }
        };

        self.token_service.delete_with_conn(conn, token).await?;

        user.data.status = AuthAccountStatus::Active;
        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn generate_reset_password_token(
        &self,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        self.c3p0.transaction(async |conn| { self.generate_reset_password_token_with_conn(conn, username).await }).await
    }

    pub async fn generate_reset_password_token_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsError> {
        info!("Generate reset password token for username [{}]", username);

        let user = self.auth_repo.fetch_by_username(conn, username).await?;

        match &user.data.status {
            AuthAccountStatus::Active => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{username}] not in status Active"),
                    code: ErrorCodes::INACTIVE_USER,
                })
            }
        };

        let token =
            self.token_service.generate_and_save_token_with_conn(conn, username, TokenType::ResetPassword).await?;

        Ok((user, token))
    }

    pub async fn reset_password_by_token(
        &self,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LsError> {
        self.c3p0
            .transaction(async |conn| { self.reset_password_by_token_with_conn(conn, reset_password_dto).await })
            .await
    }

    pub async fn reset_password_by_token_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Reset password called with token [{}]", reset_password_dto.token);
        Validator::validate(&reset_password_dto)?;

        let token = self.token_service.fetch_by_token_with_conn(conn, &reset_password_dto.token, false).await?;

        info!("Reset password of user [{}]", token.data.username);

        Validator::validate(&|error_details: &mut ErrorDetails| {
            match &token.data.token_type {
                TokenType::ResetPassword => {}
                _ => error_details.add_detail("token_type", WRONG_TYPE),
            };
            Ok(())
        })?;

        let mut user = self.auth_repo.fetch_by_username(conn, &token.data.username).await?;

        match &user.data.status {
            AuthAccountStatus::Active => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{}] not in status Active", token.data.username),
                    code: ErrorCodes::INACTIVE_USER,
                })
            }
        };

        self.token_service.delete_with_conn(conn, token).await?;

        user.data.password = self.password_service.hash_password(&reset_password_dto.password)?;
        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn change_password(&self, dto: ChangePasswordDto) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.change_password_with_conn(conn, dto).await }).await
    }

    pub async fn change_password_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        dto: ChangePasswordDto,
    ) -> Result<AuthAccountModel, LsError> {
        info!("Reset password of user_id [{:?}]", dto.user_id);

        Validator::validate(&dto)?;

        let mut user = self.auth_repo.fetch_by_id(conn, &dto.user_id).await?;
        info!("Change password of user [{}]", user.data.username);

        match &user.data.status {
            AuthAccountStatus::Active => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{}] not in status Active", user.data.username),
                    code: ErrorCodes::INACTIVE_USER,
                })
            }
        };

        if !self.password_service.verify_match(&dto.old_password, &user.data.password)? {
            return Err(LsError::BadRequest {
                message: "Wrong credentials".to_owned(),
                code: ErrorCodes::WRONG_CREDENTIALS,
            });
        }

        user.data.password = self.password_service.hash_password(&dto.new_password)?;

        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn fetch_by_user_id(&self, user_id: &u64) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.fetch_by_user_id_with_conn(conn, user_id).await }).await
    }

    pub async fn fetch_by_user_id_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Fetch user with user_id [{:?}]", user_id);
        self.auth_repo.fetch_by_id(conn, user_id).await
    }

    pub async fn fetch_by_username(&self, username: &str) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.fetch_by_username_with_conn(conn, username).await }).await
    }

    pub async fn fetch_by_username_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Fetch user with username [{}]", username);
        self.auth_repo.fetch_by_username(conn, username).await
    }

    pub async fn fetch_all_by_status(
        &self,
        status: AuthAccountStatus,
        start_user_id: &u64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError> {
        self.c3p0
            .transaction(async |conn| { self.fetch_all_by_status_with_conn(conn, status, start_user_id, limit).await })
            .await
    }

    pub async fn fetch_all_by_status_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        status: AuthAccountStatus,
        start_user_id: &u64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError> {
        debug!("Fetch all with status [{}], start_user_id {:?}, limit {}", status, start_user_id, limit);
        self.auth_repo.fetch_all_by_status(conn, status, start_user_id, limit).await
    }

    pub async fn add_roles(&self, user_id: &u64, roles: &[String]) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.add_roles_with_conn(conn, user_id, roles).await }).await
    }

    pub async fn add_roles_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsError> {
        info!("Add roles [{:?}] to user_id [{:?}]", roles, user_id);

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            if !account.data.roles.contains(role) {
                account.data.roles.push(role.to_owned())
            }
        }
        self.auth_repo.update(conn, account).await
    }

    pub async fn delete_roles(&self, user_id: &u64, roles: &[String]) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.delete_roles_with_conn(conn, user_id, roles).await }).await
    }

    pub async fn delete_roles_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsError> {
        info!("delete roles [{:?}] to user_id [{:?}]", roles, user_id);

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            account.data.roles.retain(|r| r != role);
        }
        self.auth_repo.update(conn, account).await
    }

    pub async fn change_user_data(
        &self,
        user_id: &u64,
        new_username: Option<String>,
        new_email: Option<String>,
    ) -> Result<AuthAccountModel, LsError> {
        self.c3p0
            .transaction(async |conn| { self.change_user_data_with_conn(conn, user_id, new_username, new_email).await })
            .await
    }

    pub async fn change_user_data_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
        new_username: Option<String>,
        new_email: Option<String>,
    ) -> Result<AuthAccountModel, LsError> {
        info!(
            "Change user data of user_id [{:?}]. New username: [{:?}]. New email: [{:?}]",
            user_id, new_username, new_email
        );

        let mut user = self.auth_repo.fetch_by_id(conn, user_id).await?;

        if let Some(username) = new_username {
            info!(
                "Change user data of user_id [{:?}]. Old username: [{}] New username: [{}]",
                user_id, user.data.username, username
            );
            user.data.username = username;
        }

        if let Some(email) = new_email {
            info!(
                "Change user data of user_id [{:?}]. Old email: [{}] New email: [{}]",
                user_id, user.data.email, email
            );
            user.data.email = email;
        }

        self.auth_repo.update(conn, user).await
    }

    pub async fn disable_by_user_id(&self, user_id: &u64) -> Result<AuthAccountModel, LsError> {
        self.c3p0.transaction(async |conn| { self.disable_by_user_id_with_conn(conn, user_id).await }).await
    }

    pub async fn disable_by_user_id_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Disable user with user_id [{:?}]", user_id);
        let mut user = self.auth_repo.fetch_by_id(conn, user_id).await?;

        match &user.data.status {
            AuthAccountStatus::Active => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{user_id:?}] not in status Active"),
                    code: ErrorCodes::INACTIVE_USER,
                })
            }
        };

        user.data.status = AuthAccountStatus::Disabled;
        self.auth_repo.update(conn, user).await
    }

    pub async fn reactivate_disabled_user_by_user_id(&self, user_id: &u64) -> Result<AuthAccountModel, LsError> {
        self.c3p0
            .transaction(async |conn| { self.reactivate_disabled_user_by_user_id_with_conn(conn, user_id).await })
            .await
    }

    pub async fn reactivate_disabled_user_by_user_id_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        user_id: &u64,
    ) -> Result<AuthAccountModel, LsError> {
        debug!("Reactivate disabled user with user_id [{:?}]", user_id);
        let mut user = self.auth_repo.fetch_by_id(conn, user_id).await?;

        match &user.data.status {
            AuthAccountStatus::Disabled => {}
            _ => {
                return Err(LsError::BadRequest {
                    message: format!("User [{user_id:?}] not in status Disabled"),
                    code: ErrorCodes::ACTIVE_USER,
                })
            }
        };

        user.data.status = AuthAccountStatus::Active;
        self.auth_repo.update(conn, user).await
    }

    pub async fn delete_by_user_id(&self, user_id: &u64) -> Result<u64, LsError> {
        self.c3p0.transaction(async |conn| { self.delete_by_user_id_with_conn(conn, user_id).await }).await
    }

    pub async fn delete_by_user_id_with_conn(&self, conn: &mut RepoManager::Tx<'_>, user_id: &u64) -> Result<u64, LsError> {
        debug!("Delete user with user_id [{:?}]", user_id);
        self.auth_repo.delete_by_id(conn, user_id).await
    }
}
