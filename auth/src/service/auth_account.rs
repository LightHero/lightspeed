use crate::config::AuthConfig;
use crate::repository::auth_account::AuthAccountRepository;
use crate::service::token::TokenService;
use crate::service::password_codec::PasswordCodecService;
use lightspeed_core::error::LightSpeedError;
use crate::PoolManager;
use c3p0::*;
use crate::model::auth_account::AuthAccountStatus;
use lightspeed_core::service::auth::Auth;

#[derive(Clone)]
pub struct AuthAccountService {
    c3p0: C3p0Pool<PoolManager>,
    auth_config: AuthConfig,
    auth_repo: AuthAccountRepository,
    password_service: PasswordCodecService,
    token_service: TokenService,
}

impl AuthAccountService {

    pub fn new(c3p0: C3p0Pool<PoolManager>, auth_config: AuthConfig, token_service: TokenService, password_service: PasswordCodecService, auth_repo: AuthAccountRepository) -> Self {
        AuthAccountService {
            c3p0,
            auth_config,
            auth_repo,
            password_service,
            token_service,
        }
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Auth, LightSpeedError> {
        let model =
            self.auth_repo.fetch_by_username(&self.c3p0.connection()?, username)?
            .filter(|model| match model.data.status {
                AuthAccountStatus::Active => true,
                _ => false
            });

        if let Some(user) = model {
            if self.password_service.verify_match(password, &user.data.password)? {
                return Ok(Auth{
                    username: user.data.username,
                    id: user.id,
                    roles: user.data.roles
                });
            }
        };

        Err(LightSpeedError::BadRequest {message: format!("")})

    }

    pub fn create_user(&self) -> Result<(), LightSpeedError> {
        unimplemented!()
    }
}