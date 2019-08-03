use crate::model::token::{TokenModel, TokenType, TokenData};
use crate::repository::token::{TokenRepository};
use c3p0::*;
use ls_core::config::UIConfig;
use ls_core::error::LightSpeedError;
use ls_core::utils::current_epoch_seconds;
use crate::config::AuthConfig;

#[derive(Clone)]
pub struct TokenService {
    ui_config: UIConfig,
    auth_config: AuthConfig,
    token_repo: TokenRepository,
}

impl TokenService {

    pub fn new(auth_config: AuthConfig, ui_config: UIConfig, token_repo: TokenRepository) -> Self {
        TokenService { auth_config, ui_config, token_repo }
    }

    pub fn generate_and_save_token(&self, conn: &PgConnection, username: String, token_type: TokenType) -> Result<TokenModel, LightSpeedError> {
        let issued_at = current_epoch_seconds();
        let expire_at_epoch = issued_at + (self.auth_config.token_activation_validity_minutes * 60);
        let token = NewModel::new(TokenData{
            token: "".to_owned(),
            token_type,
            username,
            expire_at_epoch
        });
        Ok(self.token_repo.save(conn, token)?)
    }

    pub fn generate_public_token_url(&self, token: &TokenModel) -> String {
        match &token.data.token_type {
            TokenType::AccountActivation => {
                format!("{}{}{}", self.ui_config.public_domain, self.auth_config.activation_token_ui_url, token.data.token)
            }
            TokenType::ResetPassword => {
                format!("{}{}{}", self.ui_config.public_domain, self.auth_config.reset_password_token_ui_url, token.data.token)
            }
        }
    }

    pub fn fetch_by_token(&self, conn: &PgConnection, token: &str, validate: bool) -> Result<Option<TokenModel>, LightSpeedError> {
        let token_model = self.token_repo.fetch_by_token(conn, token)?;
        /*
        if validate {
            return tokenValidator
            .validateThrowException(tokenRepository.fetchByToken(dsl, token))
            }
        */
        return Ok(token_model)
    }

    pub fn delete(&self, conn: &PgConnection, token_model: TokenModel) -> Result<u64, LightSpeedError> {
        Ok(self.token_repo.delete(conn, &token_model)?)
    }

}
