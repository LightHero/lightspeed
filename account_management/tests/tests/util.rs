use lightspeed_account_management::LsAuthModule;
use lightspeed_account_management::dto::create_login_dto::CreateLoginDto;
use lightspeed_account_management::error::LsAccountManagerError;
use lightspeed_account_management::model::auth_account::AuthAccountModel;
use lightspeed_account_management::model::token::TokenModel;
use lightspeed_account_management::repository::AuthRepositoryManager;
use lightspeed_core::model::language::Language;
use lightspeed_core::utils::new_hyphenated_uuid;
use std::collections::HashMap;

pub async fn create_user<RepoManager: AuthRepositoryManager>(
    auth_module: &LsAuthModule<RepoManager>,
    activate: bool,
) -> Result<(AuthAccountModel, TokenModel), LsAccountManagerError> {
    create_user_with_password(auth_module, &new_hyphenated_uuid(), activate).await
}

pub async fn create_user_with_password<RepoManager: AuthRepositoryManager>(
    auth_module: &LsAuthModule<RepoManager>,
    password: &str,
    activate: bool,
) -> Result<(AuthAccountModel, TokenModel), LsAccountManagerError> {
    let username = new_hyphenated_uuid();
    let email = format!("{username}@email.fake");

    let (user, token) = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username),
            email,
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::En,
            password: password.to_string(),
            password_confirm: password.to_string(),
        })
        .await?;

    if activate {
        let activated_user = auth_module.auth_account_service.activate_user(&token.data.token).await?;
        Ok((activated_user, token))
    } else {
        Ok((user, token))
    }
}
