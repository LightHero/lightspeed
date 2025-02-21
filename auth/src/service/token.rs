use crate::config::AuthConfig;
use crate::model::token::{TokenData, TokenModel, TokenType};
use crate::repository::{AuthRepositoryManager, TokenRepository};
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::service::validator::Validator;
use lightspeed_core::utils::*;
use log::*;

#[derive(Clone)]
pub struct LsTokenService<RepoManager: AuthRepositoryManager> {
    auth_config: AuthConfig,
    token_repo: RepoManager::TokenRepo,
}

impl<RepoManager: AuthRepositoryManager> LsTokenService<RepoManager> {
    pub fn new(auth_config: AuthConfig, token_repo: RepoManager::TokenRepo) -> Self {
        LsTokenService { auth_config, token_repo }
    }

    pub async fn generate_and_save_token_with_conn<S: Into<String>>(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: S,
        token_type: TokenType,
    ) -> Result<TokenModel, LsError> {
        let username = username.into();
        info!("Generate and save token of type [{:?}] for username [{}]", token_type, username);

        let issued_at = current_epoch_seconds();
        let expire_at_epoch = issued_at + (self.auth_config.activation_token_validity_minutes * 60);
        let token = NewModel::new(TokenData {
            token: new_hyphenated_uuid(),
            token_type,
            username,
            expire_at_epoch_seconds: expire_at_epoch,
        });
        self.token_repo.save(conn, token).await
    }

    pub async fn fetch_by_token_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        token: &str,
        validate: bool,
    ) -> Result<TokenModel, LsError> {
        debug!("Fetch by token [{}]", token);
        let token_model = self.token_repo.fetch_by_token(conn, token).await?;

        if validate {
            Validator::validate(&token_model.data)?;
        };

        Ok(token_model)
    }

    pub async fn fetch_all_by_username_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        username: &str,
    ) -> Result<Vec<TokenModel>, LsError> {
        debug!("Fetch by username [{}]", username);
        self.token_repo.fetch_by_username(conn, username).await
    }

    pub async fn delete_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        token_model: TokenModel,
    ) -> Result<TokenModel, LsError> {
        debug!("Delete token_model with id [{:?}] and token [{}]", token_model.id, token_model.data.token);
        self.token_repo.delete(conn, token_model).await
    }
}
