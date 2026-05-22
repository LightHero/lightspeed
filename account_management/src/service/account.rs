use crate::config::AMConfig;
use crate::dto::change_password_dto::ChangePasswordDto;
use crate::dto::create_login_dto::CreateLoginDto;
use crate::dto::reset_password_dto::ResetPasswordDto;
use crate::error::LsAccountManagementError;
use crate::model::auth_account::{AccountData, AccountStatus, AuthAccountModel};
use crate::model::token::{TokenModel, TokenType};
use crate::repository::{AMRepositoryManager, AccountRepository};
use crate::service::password_codec::LsPasswordCodecService;
use crate::service::token::LsTokenService;
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::service::auth::Auth;
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use std::sync::Arc;

pub const WRONG_TYPE: &str = "WRONG_TYPE";

#[derive(Clone)]
pub struct LsAMAccountService<RepoManager: AMRepositoryManager> {
    c3p0: RepoManager::C3P0,
    auth_config: AMConfig,
    auth_repo: RepoManager::AccountRepo,
    password_service: Arc<LsPasswordCodecService>,
    token_service: Arc<LsTokenService<RepoManager>>,
}

impl<RepoManager: AMRepositoryManager> LsAMAccountService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        auth_config: AMConfig,
        token_service: Arc<LsTokenService<RepoManager>>,
        password_service: Arc<LsPasswordCodecService>,
        auth_repo: RepoManager::AccountRepo,
    ) -> Self {
        LsAMAccountService { c3p0, auth_config, auth_repo, password_service, token_service }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Auth, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.login_with_conn(conn, username, password).await).await
    }

    pub async fn login_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
        password: &str,
    ) -> Result<Auth, LsAccountManagementError> {
        debug!("login attempt with username [{username}]");
        let model = self.auth_repo.fetch_by_username_optional(conn, username).await?;

        if let Some(user) = model
            && self.password_service.verify_match(password, &user.data.password).await?
        {
            match &user.data.status {
                AccountStatus::Active => {}
                _ => {
                    return Err(LsAccountManagementError::InactiveUser(username.to_string()));
                }
            };

            let creation_ts_seconds = current_epoch_seconds();

            if let Some(expiration_secs) = self.auth_config.password_expiration_seconds {
                let password_set_at = user.data.password_updated_date_epoch_seconds;
                if creation_ts_seconds.saturating_sub(password_set_at) >= expiration_secs as i64 {
                    return Err(LsAccountManagementError::ExpiredPassword(username.to_string()));
                }
            }

            let expiration_ts_seconds =
                creation_ts_seconds + (self.auth_config.auth_session_max_validity_minutes as i64 * 60);

            return Ok(Auth::new(
                user.id,
                user.data.username,
                user.data.roles,
                creation_ts_seconds,
                expiration_ts_seconds,
            ));
        } else {
            // Even out timing between "no such user" and "wrong password" to
            // prevent username enumeration via response time.
            let _ = self.password_service.verify_match(password, self.password_service.dummy_hash()).await;
        };

        Err(LsAccountManagementError::WrongCredentials)
    }

    pub async fn create_user(
        &self,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.create_user_with_conn(conn, create_login_dto).await).await
    }

    pub async fn create_user_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        create_login_dto: CreateLoginDto,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        info!(
            "Create login attempt with username [{:?}] and email [{}]",
            create_login_dto.username, create_login_dto.email
        );
        let hashed_password = self.password_service.hash_password(&create_login_dto.password).await?;

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

        if let Some(_existing_user) = self.auth_repo.fetch_by_username_optional(conn, &username).await? {
            return Err(LsAccountManagementError::UsernameAlreadyUsed);
        }
        if let Some(_existing_email) = self.auth_repo.fetch_by_email_optional(conn, &create_login_dto.email).await? {
            return Err(LsAccountManagementError::EmailAlreadyUsed);
        }

        let now = current_epoch_seconds();
        let auth_account_model = self
            .auth_repo
            .save(
                conn,
                NewRecord::new(AccountData {
                    username,
                    email: create_login_dto.email,
                    password: hashed_password,
                    roles: self.auth_config.default_roles_on_account_creation.clone(),
                    created_date_epoch_seconds: now,
                    password_updated_date_epoch_seconds: now,
                    status: AccountStatus::PendingActivation,
                }),
            )
            .await?;

        let token = self.generate_activation_token_with_conn(conn, &auth_account_model.data.username).await?;
        Ok((auth_account_model, token))
    }

    async fn generate_activation_token_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
    ) -> Result<TokenModel, LsAccountManagementError> {
        debug!("Generate activation token for username [{username}]");
        self.token_service.generate_and_save_token_with_conn(conn, username, TokenType::AccountActivation).await
    }

    pub async fn generate_new_activation_token_by_username_and_email(
        &self,
        username: &str,
        email: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        self.c3p0
            .transaction(async |conn| {
                self.generate_new_activation_token_by_username_and_email_with_conn(conn, username, email).await
            })
            .await
    }

    pub async fn generate_new_activation_token_by_username_and_email_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
        email: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        debug!("Generate new activation token for username [{username}] and email [{email}]");

        // All "this request is not eligible" branches — username unknown,
        // email mismatch, account not in PendingActivation — collapse into
        // a single error with one shared code and message. The previous
        // implementation returned three distinguishable responses:
        // `LsError::NotFound` (unknown user), `ValidationError` with
        // `WRONG_EMAIL` (email mismatch), and `BadRequest(NOT_PENDING_USER)`
        // (status mismatch). That difference made the endpoint a textbook
        // account-enumeration oracle: an attacker probing username/email
        // pairs could distinguish "this user exists" from "this email is
        // theirs" from "their account is already active". We now log the
        // precise reason at debug level for ops/forensics and surface a
        // single opaque error to the caller.
        let generic_failure = || LsAccountManagementError::WrongCredentials;

        let user = match self.auth_repo.fetch_by_username_optional(conn, username).await? {
            Some(u) => u,
            None => {
                debug!("generate_new_activation_token: username [{username}] not found");
                return Err(generic_failure());
            }
        };

        if user.data.email != email {
            debug!("generate_new_activation_token: email mismatch for username [{username}]");
            return Err(generic_failure());
        }

        if !matches!(user.data.status, AccountStatus::PendingActivation) {
            debug!("generate_new_activation_token: username [{username}] not in PendingActivation");
            return Err(generic_failure());
        }

        // Invalidate every previous activation token so an attacker cannot
        // re-use one that leaked. Reset-password tokens belong to a different
        // lifecycle and are left untouched.
        let existing_tokens = self.token_service.fetch_all_by_username_with_conn(conn, username).await?;
        for token in existing_tokens.into_iter().filter(|t| t.data.token_type == TokenType::AccountActivation) {
            self.token_service.delete_with_conn(conn, token).await?;
        }

        info!("Send new activation token to user [{username}]");
        let token = self.generate_activation_token_with_conn(conn, &user.data.username).await?;
        Ok((user, token))
    }

    pub async fn activate_user(&self, activation_token: &str) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.activate_user_with_conn(conn, activation_token).await).await
    }

    pub async fn activate_user_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        activation_token: &str,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Activate user called with token [{activation_token}]");

        let token = self.token_service.fetch_by_token_with_conn(conn, activation_token, true).await?;

        match &token.data.token_type {
            TokenType::AccountActivation => {}
            _ => return Err(LsAccountManagementError::TokenNotValid),
        };

        info!("Activate user [{}]", token.data.username);

        let mut user = self.auth_repo.fetch_by_username(conn, &token.data.username).await?;

        match &user.data.status {
            AccountStatus::PendingActivation => {}
            _ => {
                return Err(LsAccountManagementError::UserNotPendingActivation);
            }
        };

        self.token_service.delete_with_conn(conn, token).await?;

        user.data.status = AccountStatus::Active;
        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn generate_reset_password_token(
        &self,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.generate_reset_password_token_with_conn(conn, username).await).await
    }

    pub async fn generate_reset_password_token_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
    ) -> Result<(AuthAccountModel, TokenModel), LsAccountManagementError> {
        info!("Generate reset password token for username [{username}]");

        let user = self.auth_repo.fetch_by_username(conn, username).await?;

        match &user.data.status {
            AccountStatus::Active => {}
            _ => {
                return Err(LsAccountManagementError::InactiveUser(username.to_string()));
            }
        };

        let token =
            self.token_service.generate_and_save_token_with_conn(conn, username, TokenType::ResetPassword).await?;

        Ok((user, token))
    }

    pub async fn reset_password_by_token(
        &self,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.reset_password_by_token_with_conn(conn, reset_password_dto).await).await
    }

    pub async fn reset_password_by_token_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        reset_password_dto: ResetPasswordDto,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Reset password called with token [{}]", reset_password_dto.token);

        // Validate expiry: an expired reset-password token must not be usable
        // to reset the password.
        let token = self.token_service.fetch_by_token_with_conn(conn, &reset_password_dto.token, true).await?;

        info!("Reset password of user [{}]", token.data.username);

        match &token.data.token_type {
            TokenType::ResetPassword => {}
            _ => return Err(LsAccountManagementError::TokenNotValid),
        };

        let mut user = self.auth_repo.fetch_by_username(conn, &token.data.username).await?;

        match &user.data.status {
            AccountStatus::Active => {}
            _ => {
                return Err(LsAccountManagementError::InactiveUser(token.data.username.to_string()));
            }
        };

        self.token_service.delete_with_conn(conn, token).await?;

        user.data.password = self.password_service.hash_password(&reset_password_dto.password).await?;
        user.data.password_updated_date_epoch_seconds = current_epoch_seconds();
        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn change_password(&self, dto: ChangePasswordDto) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.change_password_with_conn(conn, dto).await).await
    }

    pub async fn change_password_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        dto: ChangePasswordDto,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        info!("Reset password of user_id [{:?}]", dto.user_id);

        let mut user = self.auth_repo.fetch_by_id(conn, dto.user_id).await?;
        info!("Change password of user [{}]", user.data.username);

        match &user.data.status {
            AccountStatus::Active => {}
            _ => {
                return Err(LsAccountManagementError::InactiveUser(user.data.username.to_string()));
            }
        };

        if !self.password_service.verify_match(&dto.old_password, &user.data.password).await? {
            return Err(LsAccountManagementError::WrongCredentials);
        }

        user.data.password = self.password_service.hash_password(&dto.new_password).await?;
        user.data.password_updated_date_epoch_seconds = current_epoch_seconds();

        user = self.auth_repo.update(conn, user).await?;
        Ok(user)
    }

    pub async fn fetch_by_user_id(&self, user_id: i64) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.fetch_by_user_id_with_conn(conn, user_id).await).await
    }

    pub async fn fetch_by_user_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Fetch user with user_id [{user_id:?}]");
        self.auth_repo.fetch_by_id(conn, user_id).await
    }

    pub async fn fetch_by_username(&self, username: &str) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.fetch_by_username_with_conn(conn, username).await).await
    }

    pub async fn fetch_by_username_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Fetch user with username [{username}]");
        self.auth_repo.fetch_by_username(conn, username).await
    }

    pub async fn fetch_all_by_status(
        &self,
        status: AccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsAccountManagementError> {
        self.c3p0
            .transaction(async |conn| self.fetch_all_by_status_with_conn(conn, status, start_user_id, limit).await)
            .await
    }

    pub async fn fetch_all_by_status_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        status: AccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsAccountManagementError> {
        debug!("Fetch all with status [{status}], start_user_id {start_user_id:?}, limit {limit}");
        self.auth_repo.fetch_all_by_status(conn, status, start_user_id, limit).await
    }

    pub async fn add_roles(
        &self,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.add_roles_with_conn(conn, user_id, roles).await).await
    }

    pub async fn add_roles_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        info!("Add roles [{roles:?}] to user_id [{user_id:?}]");

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            if !account.data.roles.contains(role) {
                account.data.roles.push(role.to_owned())
            }
        }
        self.auth_repo.update(conn, account).await
    }

    pub async fn delete_roles(
        &self,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.delete_roles_with_conn(conn, user_id, roles).await).await
    }

    pub async fn delete_roles_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
        roles: &[String],
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        info!("delete roles [{roles:?}] to user_id [{user_id:?}]");

        let mut account = self.fetch_by_user_id_with_conn(conn, user_id).await?;
        for role in roles {
            account.data.roles.retain(|r| r != role);
        }
        self.auth_repo.update(conn, account).await
    }

    pub async fn change_user_data(
        &self,
        user_id: i64,
        new_username: Option<String>,
        new_email: Option<String>,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0
            .transaction(async |conn| self.change_user_data_with_conn(conn, user_id, new_username, new_email).await)
            .await
    }

    pub async fn change_user_data_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
        new_username: Option<String>,
        new_email: Option<String>,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        info!(
            "Change user data of user_id [{user_id:?}]. New username: [{new_username:?}]. New email: [{new_email:?}]"
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

    pub async fn disable_by_user_id(&self, user_id: i64) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.disable_by_user_id_with_conn(conn, user_id).await).await
    }

    pub async fn disable_by_user_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Disable user with user_id [{user_id:?}]");
        let mut user = self.auth_repo.fetch_by_id(conn, user_id).await?;

        match &user.data.status {
            AccountStatus::Active => {}
            _ => {
                return Err(LsAccountManagementError::InactiveUser(user.data.username.to_string()));
            }
        };

        user.data.status = AccountStatus::Disabled;
        self.auth_repo.update(conn, user).await
    }

    pub async fn reactivate_disabled_user_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.c3p0
            .transaction(async |conn| self.reactivate_disabled_user_by_user_id_with_conn(conn, user_id).await)
            .await
    }

    pub async fn reactivate_disabled_user_by_user_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        debug!("Reactivate disabled user with user_id [{user_id:?}]");
        let mut user = self.auth_repo.fetch_by_id(conn, user_id).await?;

        match &user.data.status {
            AccountStatus::Disabled => {}
            _ => return Err(LsAccountManagementError::NotDisabledUser(user.data.username.to_string())),
        };

        user.data.status = AccountStatus::Active;
        self.auth_repo.update(conn, user).await
    }

    pub async fn delete_by_user_id(&self, user_id: i64) -> Result<u64, LsAccountManagementError> {
        self.c3p0.transaction(async |conn| self.delete_by_user_id_with_conn(conn, user_id).await).await
    }

    pub async fn delete_by_user_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        user_id: i64,
    ) -> Result<u64, LsAccountManagementError> {
        debug!("Delete user with user_id [{user_id:?}]");
        self.auth_repo.delete_by_id(conn, user_id).await
    }
}
