use crate::config::AuthConfig;
use crate::model::token::{TokenData, TokenModel, TokenType};
use crate::repository::token::TokenRepository;
use c3p0::*;
use lightspeed_core::config::UIConfig;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::service::validator::Validator;
use lightspeed_core::utils::*;

#[derive(Clone)]
pub struct TokenService {
    ui_config: UIConfig,
    auth_config: AuthConfig,
    token_repo: TokenRepository,
}

impl TokenService {
    pub fn new(auth_config: AuthConfig, ui_config: UIConfig, token_repo: TokenRepository) -> Self {
        TokenService {
            auth_config,
            ui_config,
            token_repo,
        }
    }

    pub fn generate_and_save_token(
        &self,
        conn: &PgConnection,
        username: String,
        token_type: TokenType,
    ) -> Result<TokenModel, LightSpeedError> {
        let issued_at = current_epoch_seconds();
        let expire_at_epoch = issued_at + (self.auth_config.token_activation_validity_minutes * 60);
        let token = NewModel::new(TokenData {
            token: new_hyphenated_uuid(),
            token_type,
            username,
            expire_at_epoch,
        });
        Ok(self.token_repo.save(conn, token)?)
    }

    pub fn generate_public_token_url(&self, token: &TokenModel) -> String {
        generate_public_token_url(&self.auth_config, &self.ui_config, token)
    }

    pub fn fetch_by_token(
        &self,
        conn: &PgConnection,
        token: &str,
        validate: bool,
    ) -> Result<Option<TokenModel>, LightSpeedError> {
        let token_model = self.token_repo.fetch_by_token(conn, token)?;

        if validate {
            if let Some(token) = &token_model {
                Validator::validate(&token.data)?;
            }
        }
        return Ok(token_model);
    }

    pub fn delete(
        &self,
        conn: &PgConnection,
        token_model: TokenModel,
    ) -> Result<u64, LightSpeedError> {
        Ok(self.token_repo.delete(conn, &token_model)?)
    }
}

fn generate_public_token_url(
    auth_config: &AuthConfig,
    ui_config: &UIConfig,
    token: &TokenModel,
) -> String {
    match &token.data.token_type {
        TokenType::AccountActivation => format!(
            "{}{}{}",
            ui_config.public_domain, auth_config.activation_token_ui_url, token.data.token
        ),
        TokenType::ResetPassword => format!(
            "{}{}{}",
            ui_config.public_domain, auth_config.reset_password_token_ui_url, token.data.token
        ),
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_generate_activation_url() {
        // Arrange
        let domain = new_hyphenated_uuid();
        let token_ui_url = new_hyphenated_uuid();

        let auth_config = AuthConfig {
            reset_password_token_ui_url: "".to_owned(),
            activation_token_ui_url: token_ui_url.clone(),
            auth_email_account_created_recipient: "".to_owned(),
            token_activation_validity_minutes: 1,
            bcrypt_password_hash_cost: 0,
        };

        let ui_config = UIConfig {
            public_domain: domain.clone(),
        };

        let token = Model {
            id: 0,
            version: 0,
            data: TokenData {
                token: new_hyphenated_uuid(),
                token_type: TokenType::AccountActivation,
                username: "ufoscout".to_owned(),
                expire_at_epoch: 1,
            },
        };

        // Act
        let url = generate_public_token_url(&auth_config, &ui_config, &token);

        // Assert
        assert_eq!(
            format!("{}{}{}", domain, token_ui_url, token.data.token),
            url
        );
    }

    #[test]
    fn should_generate_reset_password_url() {
        // Arrange
        let domain = new_hyphenated_uuid();
        let token_ui_url = new_hyphenated_uuid();

        let auth_config = AuthConfig {
            reset_password_token_ui_url: token_ui_url.clone(),
            activation_token_ui_url: "".to_owned(),
            auth_email_account_created_recipient: "".to_owned(),
            token_activation_validity_minutes: 1,
            bcrypt_password_hash_cost: 0,
        };

        let ui_config = UIConfig {
            public_domain: domain.clone(),
        };

        let token = Model {
            id: 0,
            version: 0,
            data: TokenData {
                token: new_hyphenated_uuid(),
                token_type: TokenType::ResetPassword,
                username: "ufoscout".to_owned(),
                expire_at_epoch: 1,
            },
        };

        // Act
        let url = generate_public_token_url(&auth_config, &ui_config, &token);

        // Assert
        assert_eq!(
            format!("{}{}{}", domain, token_ui_url, token.data.token),
            url
        );
    }

}
