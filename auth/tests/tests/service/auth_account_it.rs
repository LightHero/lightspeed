use crate::data;
use crate::tests::util::{create_user, create_user_with_password};
use c3p0::*;
use lightspeed_auth::dto::change_password_dto::ChangePasswordDto;
use lightspeed_auth::dto::create_login_dto::CreateLoginDto;
use lightspeed_auth::dto::reset_password_dto::ResetPasswordDto;
use lightspeed_auth::model::auth_account::AuthAccountStatus;
use lightspeed_auth::model::token::TokenType;
use lightspeed_auth::repository::AuthRepositoryManager;
use lightspeed_auth::service::auth_account::AuthAccountService;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::model::language::Language;
use lightspeed_core::utils::new_hyphenated_uuid;
use std::collections::HashMap;

#[test]
fn should_create_pending_user() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let username = new_hyphenated_uuid();
    let email = format!("{}@email.fake", username);
    let password = new_hyphenated_uuid();

    let (user, token) = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username.clone()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        })?;

    assert_eq!(username, user.data.username);

    assert!(auth_module
        .password_codec
        .verify_match(&password, &user.data.password)?);

    assert!(user.data.roles.is_empty());

    assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);
    assert_eq!(username, token.data.username);

    assert_eq!(TokenType::ACCOUNT_ACTIVATION, token.data.token_type);

    Ok(())
}

#[test]
fn should_assign_default_roles_at_account_creation() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let username = new_hyphenated_uuid();
    let email = format!("{}@email.fake", username);
    let password = new_hyphenated_uuid();

    let mut auth_config = auth_module.auth_config.clone();
    auth_config.default_roles_on_account_creation = vec![new_hyphenated_uuid()];

    let auth_account_service = AuthAccountService::new(
        auth_module.repo_manager.c3p0().clone(),
        auth_config.clone(),
        auth_module.token_service.clone(),
        auth_module.password_codec.clone(),
        auth_module.repo_manager.auth_account_repo(),
    );

    let (user, _) = auth_account_service.create_user(CreateLoginDto {
        username: Some(username.clone()),
        email: email.clone(),
        data: HashMap::new(),
        accept_privacy_policy: true,
        language: Language::EN,
        password: password.clone(),
        password_confirm: password.clone(),
    })?;

    assert_eq!(username, user.data.username);
    assert_eq!(
        auth_config.default_roles_on_account_creation,
        user.data.roles
    );

    Ok(())
}

#[test]
fn should_return_user_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, false)?;

    let conn = &mut auth_module.repo_manager.c3p0().connection()?;
    let user_by_id = auth_module
        .auth_account_service
        .fetch_by_user_id_with_conn(conn, user.id)?;

    assert_eq!(user.data.username, user_by_id.data.username);

    Ok(())
}

#[test]
fn should_return_user_by_username() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, false)?;

    let conn = &mut auth_module.repo_manager.c3p0().connection()?;
    let user_by_id = auth_module
        .auth_account_service
        .fetch_by_username_with_conn(conn, &user.data.username)?;

    assert_eq!(user.id, user_by_id.id);

    Ok(())
}

#[test]
fn should_use_the_email_as_username_if_not_provided() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let email = format!("{}@email.fake", new_hyphenated_uuid());
    let password = new_hyphenated_uuid();

    let (user, _token) = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: None,
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        })?;

    assert_eq!(email, user.data.username);
    assert_eq!(email, user.data.email);

    assert!(auth_module
        .auth_account_service
        .fetch_by_username(&user.data.username)
        .is_ok());

    Ok(())
}

#[test]
fn should_use_the_email_as_username_if_username_is_empty() -> Result<(), Box<dyn std::error::Error>>
{
    let data = data(false);
    let auth_module = &data.0;

    let email = format!("{}@email.fake", new_hyphenated_uuid());
    let password = new_hyphenated_uuid();

    let (user, _token) = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some("".to_owned()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        })?;

    assert_eq!(email, user.data.username);
    assert_eq!(email, user.data.email);

    assert!(auth_module
        .auth_account_service
        .fetch_by_username(&user.data.username)
        .is_ok());

    Ok(())
}

#[test]
fn should_activate_user() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, token) = create_user(&auth_module, false)?;

    assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);
    assert_eq!(TokenType::ACCOUNT_ACTIVATION, token.data.token_type);

    let activated_user = auth_module
        .auth_account_service
        .activate_user(&token.data.token)?;

    assert_eq!(AuthAccountStatus::ACTIVE, activated_user.data.status);

    assert_eq!(user.data.username, activated_user.data.username);

    assert!(auth_module
        .auth_account_service
        .activate_user(&token.data.token)
        .is_err());

    let conn = &mut auth_module.repo_manager.c3p0().connection()?;

    assert!(auth_module
        .token_service
        .fetch_by_token(conn, &token.data.token, false)
        .is_err());

    Ok(())
}

#[test]
fn should_activate_user_only_if_activation_token_type() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, false)?;
    assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);

    auth_module.repo_manager.c3p0().transaction(|conn| {
        let token = auth_module.token_service.generate_and_save_token(
            conn,
            &user.data.username,
            TokenType::RESET_PASSWORD,
        )?;

        let activation_result = auth_module
            .auth_account_service
            .activate_user(&token.data.token);

        assert!(activation_result.is_err());

        Ok(())
    })
}

#[test]
fn should_activate_user_only_if_pending_activation() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, true)?;
    assert_eq!(AuthAccountStatus::ACTIVE, user.data.status);

    auth_module
        .repo_manager
        .c3p0()
        .transaction::<_, LightSpeedError, _>(|conn| {
            let token = auth_module.token_service.generate_and_save_token(
                conn,
                &user.data.username,
                TokenType::ACCOUNT_ACTIVATION,
            )?;

            let activation_result = auth_module
                .auth_account_service
                .activate_user(&token.data.token);

            assert!(activation_result.is_err());

            Ok(())
        })?;

    Ok(())
}

#[test]
fn should_regenerate_activation_token() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let (user, token) = create_user(&auth_module, false)?;

    let (new_user, new_token) = auth_module
        .auth_account_service
        .generate_new_activation_token(&token.data.token)?;
    assert_eq!(user.id, new_user.id);
    assert!(!(token.id == new_token.id));
    assert!(!(token.data.token == new_token.data.token));

    assert!(auth_module
        .auth_account_service
        .activate_user(&token.data.token)
        .is_err());

    let activated_user = auth_module
        .auth_account_service
        .activate_user(&new_token.data.token)?;

    assert_eq!(AuthAccountStatus::ACTIVE, activated_user.data.status);
    assert_eq!(user.id, activated_user.id);

    Ok(())
}

#[test]
fn should_regenerate_activation_token_even_if_token_expired(
) -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let (_, mut token) = create_user(&auth_module, false)?;
    token.data.expire_at_epoch_seconds = 0;

    let token_model = auth_module.repo_manager.token_repo().update(
        &mut auth_module.repo_manager.c3p0().connection()?,
        token.clone(),
    )?;

    assert!(auth_module
        .token_service
        .fetch_by_token(
            &mut auth_module.repo_manager.c3p0().connection()?,
            &token_model.data.token,
            true
        )
        .is_err());

    assert!(auth_module
        .auth_account_service
        .activate_user(&token_model.data.token)
        .is_err());

    let (_, new_token) = auth_module
        .auth_account_service
        .generate_new_activation_token(&token_model.data.token)?;

    assert!(auth_module
        .auth_account_service
        .activate_user(&new_token.data.token)
        .is_ok());

    Ok(())
}

#[test]
fn should_resend_activation_token_only_if_correct_token_type(
) -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let (user, _) = create_user(&auth_module, false)?;

    let token = auth_module.token_service.generate_and_save_token(
        &mut auth_module.repo_manager.c3p0().connection()?,
        &user.data.username,
        TokenType::RESET_PASSWORD,
    )?;

    assert!(auth_module
        .auth_account_service
        .generate_new_activation_token(&token.data.token)
        .is_err());

    Ok(())
}

#[test]
fn should_login_active_user() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let password = "123456789";
    let (user, _) = create_user_with_password(&auth_module, password, true)?;

    let auth = auth_module
        .auth_account_service
        .login(&user.data.username, password)?;
    assert_eq!(user.data.username, auth.username);

    Ok(())
}

#[test]
fn should_not_login_inactive_user() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let password = "123456789";
    let (user, _) = create_user_with_password(&auth_module, password, false)?;

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, password)
        .is_err());

    Ok(())
}

#[test]
fn should_not_login_with_wrong_username() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let password = "123456789";
    let (user, _) = create_user_with_password(&auth_module, password, true)?;

    assert!(auth_module
        .auth_account_service
        .login(&format!("{}_", user.data.username), password)
        .is_err());

    Ok(())
}

#[test]
fn should_not_login_with_wrong_password() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let password = "123456789";
    let (user, _) = create_user_with_password(&auth_module, password, true)?;

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &format!("{}_", password))
        .is_err());

    Ok(())
}

#[test]
fn create_user_should_fail_if_passwords_do_not_match() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let username = new_hyphenated_uuid();
    let email = format!("{}@email.fake", username);

    let result = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username.clone()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: new_hyphenated_uuid(),
            password_confirm: new_hyphenated_uuid(),
        });

    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("password"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn create_user_should_fail_if_not_valid_email() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;
    let username = new_hyphenated_uuid();
    let email = new_hyphenated_uuid();
    let password = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username.clone()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        });

    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("email"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn create_user_should_fail_if_not_accepted_privacy_policy() -> Result<(), Box<dyn std::error::Error>>
{
    let data = data(false);
    let auth_module = &data.0;
    let username = new_hyphenated_uuid();
    let email = format!("{}@email.fake", username);
    let password = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username.clone()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: false,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        });

    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("accept_privacy_policy"))
        }
        _ => assert!(false),
    };

    Ok(())
}

#[test]
fn create_user_should_fail_if_username_not_unique() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();

    let mut dto = CreateLoginDto {
        username: Some(new_hyphenated_uuid()),
        email: format!("{}@email.fake", new_hyphenated_uuid()),
        data: HashMap::new(),
        accept_privacy_policy: true,
        language: Language::EN,
        password: password.clone(),
        password_confirm: password.clone(),
    };

    assert!(auth_module
        .auth_account_service
        .create_user(dto.clone())
        .is_ok());

    dto.email = format!("{}@email.fake", new_hyphenated_uuid());

    let result = auth_module.auth_account_service.create_user(dto);
    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("username"))
        }
        _ => assert!(false),
    };

    Ok(())
}

#[test]
fn create_user_should_fail_if_email_not_unique() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();

    let mut dto = CreateLoginDto {
        username: Some(new_hyphenated_uuid()),
        email: format!("{}@email.fake", new_hyphenated_uuid()),
        data: HashMap::new(),
        accept_privacy_policy: true,
        language: Language::EN,
        password: password.clone(),
        password_confirm: password.clone(),
    };

    assert!(auth_module
        .auth_account_service
        .create_user(dto.clone())
        .is_ok());

    dto.username = Some(new_hyphenated_uuid());
    let result = auth_module.auth_account_service.create_user(dto);
    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("email"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn should_reset_password_by_token() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, true)?;

    let token = auth_module.token_service.generate_and_save_token(
        &mut auth_module.repo_manager.c3p0().connection()?,
        &user.data.username,
        TokenType::RESET_PASSWORD,
    )?;

    let password_new = new_hyphenated_uuid();

    let updated_user =
        auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: password_new.clone(),
            })?;

    assert_eq!(user.id, updated_user.id);

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password)
        .is_err());

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password_new)
        .is_ok());

    Ok(())
}

#[test]
fn should_reset_password_only_if_correct_token_type() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, true)?;

    let token = auth_module.token_service.generate_and_save_token(
        &mut auth_module.repo_manager.c3p0().connection()?,
        &user.data.username,
        TokenType::ACCOUNT_ACTIVATION,
    )?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .reset_password_by_token(ResetPasswordDto {
            password: password_new.clone(),
            token: token.data.token,
            password_confirm: password_new.clone(),
        });

    assert!(result.is_err());

    Ok(())
}

#[test]
fn should_reset_password_only_if_user_is_active() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, false)?;

    let token = auth_module.token_service.generate_and_save_token(
        &mut auth_module.repo_manager.c3p0().connection()?,
        &user.data.username,
        TokenType::RESET_PASSWORD,
    )?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .reset_password_by_token(ResetPasswordDto {
            password: password_new.clone(),
            token: token.data.token,
            password_confirm: password_new.clone(),
        });

    assert!(result.is_err());
    Ok(())
}

#[test]
fn should_reset_password_only_if_passwords_match() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, false)?;

    let token = auth_module.token_service.generate_and_save_token(
        &mut auth_module.repo_manager.c3p0().connection()?,
        &user.data.username,
        TokenType::RESET_PASSWORD,
    )?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .reset_password_by_token(ResetPasswordDto {
            password: password_new.clone(),
            token: token.data.token,
            password_confirm: format!("{}_", password_new.clone()),
        });

    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("password"))
        }
        _ => assert!(false),
    };

    Ok(())
}

#[test]
fn should_generate_reset_password_token() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, true)?;

    let (new_user, token) = auth_module
        .auth_account_service
        .generate_reset_password_token(&user.data.username)?;
    assert_eq!(user.id, new_user.id);
    assert_eq!(TokenType::RESET_PASSWORD, token.data.token_type);
    Ok(())
}

#[test]
fn should_not_generate_reset_password_token_if_user_not_active(
) -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let (user, _) = create_user(&auth_module, false)?;

    assert!(auth_module
        .auth_account_service
        .generate_reset_password_token(&user.data.username)
        .is_err());

    Ok(())
}

#[test]
fn should_change_user_password() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, true)?;

    let password_new = new_hyphenated_uuid();

    let updated_user = auth_module
        .auth_account_service
        .change_password(ChangePasswordDto {
            user_id: user.id,
            old_password: password.clone(),
            new_password: password_new.clone(),
            new_password_confirm: password_new.clone(),
        })?;

    assert_eq!(updated_user.id, user.id);

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password)
        .is_err());

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password_new)
        .is_ok());

    Ok(())
}

#[test]
fn should_not_change_user_password_if_wrong_old_password() -> Result<(), Box<dyn std::error::Error>>
{
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, true)?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .change_password(ChangePasswordDto {
            user_id: user.id,
            old_password: format!("__{}__", password),
            new_password: password_new.clone(),
            new_password_confirm: password_new.clone(),
        });

    assert!(result.is_err());

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password)
        .is_ok());

    assert!(auth_module
        .auth_account_service
        .login(&user.data.username, &password_new)
        .is_err());

    Ok(())
}

#[test]
fn should_not_change_user_password_if_inactive_user() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, false)?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .change_password(ChangePasswordDto {
            user_id: user.id,
            old_password: password,
            new_password: password_new.clone(),
            new_password_confirm: password_new.clone(),
        });

    assert!(result.is_err());

    Ok(())
}

#[test]
fn should_not_change_user_password_if_new_passwords_do_not_match(
) -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let password = new_hyphenated_uuid();
    let (user, _) = create_user_with_password(&auth_module, &password, true)?;

    let password_new = new_hyphenated_uuid();

    let result = auth_module
        .auth_account_service
        .change_password(ChangePasswordDto {
            user_id: user.id,
            old_password: password,
            new_password: password_new.clone(),
            new_password_confirm: format!("__{}", password_new.clone()),
        });

    assert!(result.is_err());

    match &result {
        Err(LightSpeedError::ValidationError { details }) => {
            assert!(details.details.contains_key("new_password"))
        }
        _ => assert!(false),
    }

    Ok(())
}

#[test]
fn should_add_and_remove_roles() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let auth_module = &data.0;

    let auh_service = &auth_module.auth_account_service;

    let username = new_hyphenated_uuid();
    let email = format!("{}@email.fake", username);
    let password = new_hyphenated_uuid();

    let (user, _) = auth_module
        .auth_account_service
        .create_user(CreateLoginDto {
            username: Some(username.clone()),
            email: email.clone(),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::EN,
            password: password.clone(),
            password_confirm: password.clone(),
        })?;

    assert!(user.data.roles.is_empty());

    let user = auh_service.add_roles(user.id, &vec![])?;
    assert!(user.data.roles.is_empty());

    let user = auh_service.delete_roles(user.id, &vec!["one".to_owned()])?;
    assert!(user.data.roles.is_empty());

    let user = auh_service.add_roles(user.id, &vec!["one".to_owned()])?;
    assert_eq!(vec!["one".to_owned()], user.data.roles);

    let user = auh_service.add_roles(user.id, &vec!["two".to_owned(), "three".to_owned()])?;
    assert_eq!(
        vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
        user.data.roles
    );

    let user = auh_service.delete_roles(user.id, &vec!["two".to_owned(), "four".to_owned()])?;
    assert_eq!(vec!["one".to_owned(), "three".to_owned()], user.data.roles);

    let user = auh_service.delete_roles(user.id, &vec!["one".to_owned(), "three".to_owned()])?;
    assert!(user.data.roles.is_empty());

    Ok(())
}
