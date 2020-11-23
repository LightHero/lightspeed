use crate::tests::util::{create_user, create_user_with_password};
use crate::{data, test};
use c3p0::*;
use lightspeed_auth::dto::change_password_dto::ChangePasswordDto;
use lightspeed_auth::dto::create_login_dto::CreateLoginDto;
use lightspeed_auth::dto::reset_password_dto::ResetPasswordDto;
use lightspeed_auth::model::auth_account::AuthAccountStatus;
use lightspeed_auth::model::token::TokenType;
use lightspeed_auth::repository::AuthRepositoryManager;
use lightspeed_auth::service::auth_account::AuthAccountService;
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
use lightspeed_core::model::language::Language;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};
use std::collections::HashMap;

#[test]
fn should_create_pending_user() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await?;

        assert_eq!(username, user.data.username);

        assert!(auth_module.password_codec.verify_match(&password, &user.data.password)?);

        assert!(user.data.roles.is_empty());

        assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);
        assert_eq!(username, token.data.username);

        assert_eq!(TokenType::ACCOUNT_ACTIVATION, token.data.token_type);

        Ok(())
    })
}

#[test]
fn should_assign_default_roles_at_account_creation() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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

        let (user, _) = auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::EN,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await?;

        assert_eq!(username, user.data.username);
        assert_eq!(auth_config.default_roles_on_account_creation, user.data.roles);

        Ok(())
    })
}

#[test]
fn should_return_user_by_id() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, false).await?;

        auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                let user_by_id =
                    auth_module.auth_account_service.fetch_by_user_id_with_conn(&mut conn, user.id).await?;

                assert_eq!(user.data.username, user_by_id.data.username);

                Ok(())
            })
            .await
    })
}

#[test]
fn should_return_user_by_username() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, false).await?;

        auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                let user_by_id = auth_module
                    .auth_account_service
                    .fetch_by_username_with_conn(&mut conn, &user.data.username)
                    .await?;

                assert_eq!(user.id, user_by_id.id);

                Ok(())
            })
            .await
    })
}

#[test]
fn should_use_the_email_as_username_if_not_provided() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await?;

        assert_eq!(email, user.data.username);
        assert_eq!(email, user.data.email);

        assert!(auth_module.auth_account_service.fetch_by_username(&user.data.username).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_use_the_email_as_username_if_username_is_empty() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await?;

        assert_eq!(email, user.data.username);
        assert_eq!(email, user.data.email);

        assert!(auth_module.auth_account_service.fetch_by_username(&user.data.username).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_activate_user() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, token) = create_user(&auth_module, false).await?;

        assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);
        assert_eq!(TokenType::ACCOUNT_ACTIVATION, token.data.token_type);

        let activated_user = auth_module.auth_account_service.activate_user(&token.data.token).await?;

        assert_eq!(AuthAccountStatus::ACTIVE, activated_user.data.status);

        assert_eq!(user.data.username, activated_user.data.username);

        assert!(auth_module.auth_account_service.activate_user(&token.data.token).await.is_err());

        assert!(auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module.token_service.fetch_by_token_with_conn(&mut conn, &token.data.token, false).await
            })
            .await
            .is_err());

        Ok(())
    })
}

#[test]
fn should_activate_user_only_if_activation_token_type() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, false).await?;
        assert_eq!(AuthAccountStatus::PENDING_ACTIVATION, user.data.status);

        auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                let token = auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::RESET_PASSWORD)
                    .await?;

                let activation_result = auth_module.auth_account_service.activate_user(&token.data.token).await;

                assert!(activation_result.is_err());

                Ok(())
            })
            .await
    })
}

#[test]
fn should_activate_user_only_if_pending_activation() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, true).await?;
        assert_eq!(AuthAccountStatus::ACTIVE, user.data.status);

        auth_module
            .repo_manager
            .c3p0()
            .transaction::<_, LightSpeedError, _, _>(|mut conn| async move {
                let token = auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::ACCOUNT_ACTIVATION)
                    .await?;

                let activation_result = auth_module.auth_account_service.activate_user(&token.data.token).await;

                assert!(activation_result.is_err());

                Ok(())
            })
            .await?;

        Ok(())
    })
}

#[test]
fn should_regenerate_activation_token() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let (user, token) = create_user(&auth_module, false).await?;

        let (new_user, new_token) =
            auth_module.auth_account_service.generate_new_activation_token_by_token(&token.data.token).await?;
        assert_eq!(user.id, new_user.id);
        assert!(!(token.id == new_token.id));
        assert!(!(token.data.token == new_token.data.token));

        assert!(auth_module.auth_account_service.activate_user(&token.data.token).await.is_err());

        let activated_user = auth_module.auth_account_service.activate_user(&new_token.data.token).await?;

        assert_eq!(AuthAccountStatus::ACTIVE, activated_user.data.status);
        assert_eq!(user.id, activated_user.id);

        Ok(())
    })
}

#[test]
fn should_regenerate_activation_token_by_email_and_username() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let (user, token) = create_user(&auth_module, false).await?;

        // use wrong email
        assert!(auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&user.data.username, "email")
            .await
            .is_err());

        // use wrong username
        assert!(auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email("name", &user.data.email)
            .await
            .is_err());

        let (new_user, new_token) = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&user.data.username, &user.data.email)
            .await?;

        assert_eq!(user.id, new_user.id);
        assert!(!(token.id == new_token.id));
        assert!(!(token.data.token == new_token.data.token));

        assert!(auth_module.auth_account_service.activate_user(&token.data.token).await.is_err());

        let activated_user = auth_module.auth_account_service.activate_user(&new_token.data.token).await?;

        assert_eq!(AuthAccountStatus::ACTIVE, activated_user.data.status);
        assert_eq!(user.id, activated_user.id);

        Ok(())
    })
}

#[test]
fn should_regenerate_activation_token_even_if_token_expired() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let (_, mut token) = create_user(&auth_module, false).await?;
        token.data.expire_at_epoch_seconds = 0;

        let token_model = &auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module.repo_manager.token_repo().update(&mut conn, token.clone()).await
            })
            .await?;

        assert!(auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module.token_service.fetch_by_token_with_conn(&mut conn, &token_model.data.token, true).await
            })
            .await
            .is_err());

        assert!(auth_module.auth_account_service.activate_user(&token_model.data.token).await.is_err());

        let (_, new_token) =
            auth_module.auth_account_service.generate_new_activation_token_by_token(&token_model.data.token).await?;

        assert!(auth_module.auth_account_service.activate_user(&new_token.data.token).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_resend_activation_token_only_if_correct_token_type() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let (user, _) = create_user(&auth_module, false).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::RESET_PASSWORD)
                    .await
            })
            .await?;

        assert!(auth_module
            .auth_account_service
            .generate_new_activation_token_by_token(&token.data.token)
            .await
            .is_err());

        Ok(())
    })
}

#[test]
fn should_login_active_user() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        let auth_validity_seconds = auth_module.auth_config.auth_session_max_validity_minutes * 60;
        let before_login_ts_seconds = current_epoch_seconds();

        let auth = auth_module.auth_account_service.login(&user.data.username, password).await?;

        let after_login_ts_seconds = current_epoch_seconds();

        assert_eq!(user.data.username, auth.username);

        assert!(auth.creation_ts_seconds >= before_login_ts_seconds);
        assert!(auth.creation_ts_seconds <= after_login_ts_seconds);

        assert!(auth.expiration_ts_seconds >= before_login_ts_seconds + auth_validity_seconds);
        assert!(auth.expiration_ts_seconds <= after_login_ts_seconds + auth_validity_seconds);

        Ok(())
    })
}

#[test]
fn should_not_login_inactive_user() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, false).await?;

        let result = auth_module.auth_account_service.login(&user.data.username, password).await;

        match result {
            Err(LightSpeedError::BadRequest { code, message }) => {
                assert_eq!(ErrorCodes::INACTIVE_USER, code);
                assert_eq!(format!("User [{}] not in status Active", &user.data.username), message);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_return_wrong_credentials_on_login_of_inactive_user_with_wrong_password() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, false).await?;

        let result = auth_module.auth_account_service.login(&user.data.username, "wrong_password").await;

        match result {
            Err(LightSpeedError::BadRequest { code, message }) => {
                assert_eq!(ErrorCodes::WRONG_CREDENTIALS, code);
                assert_eq!(format!("Wrong credentials"), message);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_not_login_with_wrong_username() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&format!("{}_", user.data.username), password).await.is_err());

        Ok(())
    })
}

#[test]
fn should_not_login_with_wrong_password() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, &format!("{}_", password)).await.is_err());

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_passwords_do_not_match() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("password"))
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_not_valid_email() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("email"))
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_not_accepted_privacy_policy() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("accept_privacy_policy"))
            }
            _ => assert!(false),
        };

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_username_not_unique() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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

        assert!(auth_module.auth_account_service.create_user(dto.clone()).await.is_ok());

        dto.email = format!("{}@email.fake", new_hyphenated_uuid());

        let result = auth_module.auth_account_service.create_user(dto).await;
        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("username"))
            }
            _ => assert!(false),
        };

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_email_not_unique() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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

        assert!(auth_module.auth_account_service.create_user(dto.clone()).await.is_ok());

        dto.username = Some(new_hyphenated_uuid());
        let result = auth_module.auth_account_service.create_user(dto).await;
        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("email"))
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_reset_password_by_token() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = &create_user_with_password(&auth_module, &password, true).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::RESET_PASSWORD)
                    .await
            })
            .await?;

        let password_new = new_hyphenated_uuid();

        let updated_user = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: password_new.clone(),
            })
            .await?;

        assert_eq!(user.id, updated_user.id);

        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_err());

        assert!(auth_module.auth_account_service.login(&user.data.username, &password_new).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_reset_password_only_if_correct_token_type() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::ACCOUNT_ACTIVATION)
                    .await
            })
            .await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: password_new.clone(),
            })
            .await;

        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_reset_password_only_if_user_is_active() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, false).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::RESET_PASSWORD)
                    .await
            })
            .await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: password_new.clone(),
            })
            .await;

        assert!(result.is_err());
        Ok(())
    })
}

#[test]
fn should_reset_password_only_if_passwords_match() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, false).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(&mut conn, &user.data.username, TokenType::RESET_PASSWORD)
                    .await
            })
            .await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: format!("{}_", password_new.clone()),
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("password"))
            }
            _ => assert!(false),
        };

        Ok(())
    })
}

#[test]
fn should_generate_reset_password_token() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, true).await?;

        let (new_user, token) =
            auth_module.auth_account_service.generate_reset_password_token(&user.data.username).await?;
        assert_eq!(user.id, new_user.id);
        assert_eq!(TokenType::RESET_PASSWORD, token.data.token_type);
        Ok(())
    })
}

#[test]
fn should_not_generate_reset_password_token_if_user_not_active() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(&auth_module, false).await?;

        assert!(auth_module.auth_account_service.generate_reset_password_token(&user.data.username).await.is_err());

        Ok(())
    })
}

#[test]
fn should_change_user_password() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true).await?;

        let password_new = new_hyphenated_uuid();

        let updated_user = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: password.clone(),
                new_password: password_new.clone(),
                new_password_confirm: password_new.clone(),
            })
            .await?;

        assert_eq!(updated_user.id, user.id);

        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_err());

        assert!(auth_module.auth_account_service.login(&user.data.username, &password_new).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_not_change_user_password_if_wrong_old_password() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true).await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: format!("__{}__", password),
                new_password: password_new.clone(),
                new_password_confirm: password_new.clone(),
            })
            .await;

        assert!(result.is_err());

        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_ok());

        assert!(auth_module.auth_account_service.login(&user.data.username, &password_new).await.is_err());

        Ok(())
    })
}

#[test]
fn should_not_change_user_password_if_inactive_user() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, false).await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: password,
                new_password: password_new.clone(),
                new_password_confirm: password_new.clone(),
            })
            .await;

        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_not_change_user_password_if_new_passwords_do_not_match() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true).await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: password,
                new_password: password_new.clone(),
                new_password_confirm: format!("__{}", password_new.clone()),
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("new_password"))
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_add_and_remove_roles() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
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
            })
            .await?;

        assert!(user.data.roles.is_empty());

        let user = auh_service.add_roles(user.id, &vec![]).await?;
        assert!(user.data.roles.is_empty());

        let user = auh_service.delete_roles(user.id, &vec!["one".to_owned()]).await?;
        assert!(user.data.roles.is_empty());

        let user = auh_service.add_roles(user.id, &vec!["one".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned()], user.data.roles);

        let user = auh_service.add_roles(user.id, &vec!["two".to_owned(), "three".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned(), "two".to_owned(), "three".to_owned()], user.data.roles);

        let user = auh_service.delete_roles(user.id, &vec!["two".to_owned(), "four".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned(), "three".to_owned()], user.data.roles);

        let user = auh_service.delete_roles(user.id, &vec!["one".to_owned(), "three".to_owned()]).await?;
        assert!(user.data.roles.is_empty());

        Ok(())
    })
}

#[test]
fn should_change_username() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let new_username = new_hyphenated_uuid();
        let updated_user =
            auth_module.auth_account_service.change_user_data(user.id, Some(new_username.clone()), None).await.unwrap();

        // Assert

        assert_eq!(new_username, updated_user.data.username);
        assert_eq!(user.id, updated_user.id);
        assert_eq!(user.data.email, updated_user.data.email);
        assert_eq!(user.data.status, updated_user.data.status);
        assert_eq!(user.data.created_date_epoch_seconds, updated_user.data.created_date_epoch_seconds);
        assert_eq!(user.data.password, updated_user.data.password);

        assert!(auth_module.auth_account_service.login(&new_username, password).await.is_ok());

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        Ok(())
    })
}

#[test]
fn should_change_email() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let new_email = format!("{}@test.com", new_hyphenated_uuid());
        let updated_user =
            auth_module.auth_account_service.change_user_data(user.id, None, Some(new_email.clone())).await.unwrap();

        // Assert
        assert_eq!(user.data.username, updated_user.data.username);
        assert_eq!(user.id, updated_user.id);
        assert_eq!(new_email, updated_user.data.email);
        assert_eq!(user.data.status, updated_user.data.status);
        assert_eq!(user.data.created_date_epoch_seconds, updated_user.data.created_date_epoch_seconds);
        assert_eq!(user.data.password, updated_user.data.password);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_change_username_and_email() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let new_username = new_hyphenated_uuid();
        let new_email = format!("{}@test.com", new_username);
        let updated_user = auth_module
            .auth_account_service
            .change_user_data(user.id, Some(new_username.clone()), Some(new_email.clone()))
            .await
            .unwrap();

        // Assert
        assert_eq!(new_username, updated_user.data.username);
        assert_eq!(user.id, updated_user.id);
        assert_eq!(new_email, updated_user.data.email);
        assert_eq!(user.data.status, updated_user.data.status);
        assert_eq!(user.data.created_date_epoch_seconds, updated_user.data.created_date_epoch_seconds);
        assert_eq!(user.data.password, updated_user.data.password);

        assert!(auth_module.auth_account_service.login(&new_username, password).await.is_ok());

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        Ok(())
    })
}

#[test]
fn should_disable_an_active_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let updated_user = auth_module.auth_account_service.disable_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(AuthAccountStatus::DISABLED, updated_user.data.status);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        let loaded_user = auth_module.auth_account_service.fetch_by_user_id(user.id).await.unwrap();

        assert_eq!(AuthAccountStatus::DISABLED, loaded_user.data.status);

        Ok(())
    })
}

#[test]
fn should_fail_disabling_a_pending_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, false).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        // Act
        let result = auth_module.auth_account_service.disable_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_fail_disabling_a_disabled_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        auth_module.auth_account_service.disable_by_user_id(user.id).await.unwrap();

        // Act
        let result = auth_module.auth_account_service.disable_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_activate_a_disabled_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.disable_by_user_id(user.id).await.is_ok());

        // Act
        let updated_user = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(AuthAccountStatus::ACTIVE, updated_user.data.status);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        let loaded_user = auth_module.auth_account_service.fetch_by_user_id(user.id).await.unwrap();

        assert_eq!(AuthAccountStatus::ACTIVE, loaded_user.data.status);

        Ok(())
    })
}

#[test]
fn should_fail_reactivating_a_pending_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, false).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        // Act
        let result = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_fail_reactivating_an_active_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let result = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_delete_a_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        // Act
        let updated_user = auth_module.auth_account_service.delete_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(AuthAccountStatus::ACTIVE, updated_user.data.status);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        assert!(auth_module.auth_account_service.fetch_by_user_id(user.id).await.is_err());

        Ok(())
    })
}

#[test]
fn should_fail_deleting_a_deleted_user() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true).await?;

        auth_module.auth_account_service.delete_by_user_id(user.id).await.unwrap();

        // Act
        let result = auth_module.auth_account_service.delete_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_return_users_by_status() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;

        let (user_active_1, _) = create_user(&auth_module, true).await?;
        let (user_active_2, _) = create_user(&auth_module, true).await?;
        let (user_pending_1, _) = create_user(&auth_module, false).await?;
        let (user_pending_2, _) = create_user(&auth_module, false).await?;
        let (user_disabled_1, _) = create_user(&auth_module, true).await?;

        assert!(auth_module.auth_account_service.disable_by_user_id(user_disabled_1.id).await.is_ok());

        // Act
        let all_active_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::ACTIVE, 0, u32::max_value())
            .await
            .unwrap();

        let all_pending_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::PENDING_ACTIVATION, 0, u32::max_value())
            .await
            .unwrap();

        let all_disabled_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::DISABLED, 0, u32::max_value())
            .await
            .unwrap();

        // Assert
        assert!(!all_active_users.is_empty());
        assert!(all_active_users.iter().any(|user| user.id == user_active_1.id));
        assert!(all_active_users.iter().any(|user| user.id == user_active_2.id));
        assert!(!all_active_users.iter().any(|user| user.id == user_pending_1.id));
        assert!(!all_active_users.iter().any(|user| user.id == user_pending_2.id));
        assert!(!all_active_users.iter().any(|user| user.id == user_disabled_1.id));

        assert!(!all_pending_users.is_empty());
        assert!(!all_pending_users.iter().any(|user| user.id == user_active_1.id));
        assert!(!all_pending_users.iter().any(|user| user.id == user_active_2.id));
        assert!(all_pending_users.iter().any(|user| user.id == user_pending_1.id));
        assert!(all_pending_users.iter().any(|user| user.id == user_pending_2.id));
        assert!(!all_pending_users.iter().any(|user| user.id == user_disabled_1.id));

        assert!(!all_disabled_users.is_empty());
        assert!(!all_disabled_users.iter().any(|user| user.id == user_active_1.id));
        assert!(!all_disabled_users.iter().any(|user| user.id == user_active_2.id));
        assert!(!all_disabled_users.iter().any(|user| user.id == user_pending_1.id));
        assert!(!all_disabled_users.iter().any(|user| user.id == user_pending_2.id));
        assert!(all_disabled_users.iter().any(|user| user.id == user_disabled_1.id));

        Ok(())
    })
}

#[test]
fn should_return_users_by_status_with_offset_and_limit() -> Result<(), LightSpeedError> {
    test(async {
        // Arrange
        let data = data(true).await;
        let auth_module = &data.0;

        create_user(&auth_module, true).await?;
        create_user(&auth_module, true).await?;
        create_user(&auth_module, true).await?;

        // Act
        let all_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::ACTIVE, 0, u32::max_value())
            .await
            .unwrap();

        let offset_one_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::ACTIVE, 1, u32::max_value())
            .await
            .unwrap();

        let limit_two_users =
            auth_module.auth_account_service.fetch_all_by_status(AuthAccountStatus::ACTIVE, 1, 2).await.unwrap();

        // Assert
        assert_eq!(all_users[1].id, offset_one_users[0].id);
        assert_eq!(all_users.len() - 1, limit_two_users.len());
        assert_eq!(all_users[1].id, limit_two_users[0].id);
        assert_eq!(2, limit_two_users.len());

        Ok(())
    })
}
