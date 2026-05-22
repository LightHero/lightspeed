use crate::config::AMConfig;
use crate::error::LsAccountManagementError;
use crate::model::token::{TokenData, TokenModel, TokenType};
use crate::repository::{AMRepositoryManager, TokenRepository};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::utils::*;
use log::*;

#[derive(Clone)]
pub struct LsTokenService<RepoManager: AMRepositoryManager> {
    auth_config: AMConfig,
    token_repo: RepoManager::TokenRepo,
}

impl<RepoManager: AMRepositoryManager> LsTokenService<RepoManager> {
    pub fn new(auth_config: AMConfig, token_repo: RepoManager::TokenRepo) -> Self {
        LsTokenService { auth_config, token_repo }
    }

    pub async fn generate_and_save_token_with_conn<S: Into<String>>(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: S,
        token_type: TokenType,
    ) -> Result<TokenModel, LsAccountManagementError> {
        let username = username.into();
        info!("Generate and save token of type [{token_type:?}] for username [{username}]");

        let issued_at = current_epoch_seconds();

        // Lazy sweep: every new token write opportunistically removes every
        // already-expired token in the table. This bounds growth without
        // requiring an external scheduler. The sweep runs in the same tx as
        // the insert; touched rows are disjoint, so it does not race against
        // concurrent fetches/deletes of still-valid tokens.
        self.delete_expired_with_conn(conn, issued_at).await?;

        let expire_at_epoch = issued_at + (self.auth_config.activation_token_validity_minutes as i64 * 60);
        let token = NewRecord::new(TokenData {
            token: new_hyphenated_uuid(),
            token_type,
            username,
            expire_at_epoch_seconds: expire_at_epoch,
        });
        self.token_repo.save(conn, token).await
    }

    pub async fn delete_expired_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        threshold_epoch_seconds: i64,
    ) -> Result<u64, LsAccountManagementError> {
        let deleted = self.token_repo.delete_expired(conn, threshold_epoch_seconds).await?;
        if deleted > 0 {
            debug!("Lazy sweep removed [{deleted}] expired token(s)");
        }
        Ok(deleted)
    }

    pub async fn fetch_by_token_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        token: &str,
        fail_if_expired: bool,
    ) -> Result<TokenModel, LsAccountManagementError> {
        debug!("Fetch by token [{token}]");
        let token_model = self.token_repo.fetch_by_token(conn, token).await?;

        if fail_if_expired && current_epoch_seconds() > token_model.data.expire_at_epoch_seconds {
            Err(LsAccountManagementError::TokenExpired)
        } else {
            Ok(token_model)
        }
    }

    pub async fn fetch_all_by_username_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
    ) -> Result<Vec<TokenModel>, LsAccountManagementError> {
        debug!("Fetch by username [{username}]");
        self.token_repo.fetch_by_username(conn, username).await
    }

    pub async fn delete_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        token_model: TokenModel,
    ) -> Result<TokenModel, LsAccountManagementError> {
        debug!("Delete token_model with id [{:?}] and token [{}]", token_model.id, token_model.data.token);
        self.token_repo.delete(conn, token_model).await
    }
}
