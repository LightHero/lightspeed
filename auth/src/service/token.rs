use crate::config::AuthConfig;
use crate::model::token::{TokenData, TokenModel, TokenType};
use crate::repository::{AuthRepositoryManager, TokenRepository};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::service::validator::Validator;
use lightspeed_core::utils::*;

#[derive(Clone)]
pub struct TokenService<RepoManager: AuthRepositoryManager> {
    auth_config: AuthConfig,
    token_repo: RepoManager::TokenRepo,
}

impl<RepoManager: AuthRepositoryManager> TokenService<RepoManager> {
    pub fn new(auth_config: AuthConfig, token_repo: RepoManager::TokenRepo) -> Self {
        TokenService {
            auth_config,
            token_repo,
        }
    }

    pub async fn generate_and_save_token<S: Into<String>>(
        &self,
        conn: &mut RepoManager::Conn,
        username: S,
        token_type: TokenType,
    ) -> Result<TokenModel, LightSpeedError> {
        let issued_at = current_epoch_seconds();
        let expire_at_epoch = issued_at + (self.auth_config.token_validity_minutes * 60);
        let token = NewModel::new(TokenData {
            token: new_hyphenated_uuid(),
            token_type,
            username: username.into(),
            expire_at_epoch_seconds: expire_at_epoch,
        });
        self.token_repo.save(conn, token).await
    }

    pub async fn fetch_by_token(
        &self,
        conn: &mut RepoManager::Conn,
        token: &str,
        validate: bool,
    ) -> Result<TokenModel, LightSpeedError> {
        let token_model = self.token_repo.fetch_by_token(conn, token).await?;

        if validate {
            Validator::validate(&token_model.data)?;
        };

        Ok(token_model)
    }

    pub async fn delete(
        &self,
        conn: &mut RepoManager::Conn,
        token_model: TokenModel,
    ) -> Result<TokenModel, LightSpeedError> {
        self.token_repo.delete(conn, token_model).await
    }
}
