use crate::config::AuthConfig;
use crate::repository::auth_account::AuthAccountRepository;
use crate::service::token::TokenService;

#[derive(Clone)]
pub struct AuthAccountService {
    auth_config: AuthConfig,
    token_service: TokenService,
    auth_repo: AuthAccountRepository
}

impl AuthAccountService {

    pub fn new(auth_config: AuthConfig, token_service: TokenService, auth_repo: AuthAccountRepository) -> Self {
        AuthAccountService {
            auth_config,
            auth_repo,
            token_service,
        }
    }

}