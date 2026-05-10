use crate::data;
use crate::tests::util::{create_user, create_user_with_password};
use c3p0::*;
use lightspeed_auth::config::AuthConfig;
use lightspeed_auth::dto::ERR_PASSWORD_TOO_SHORT;
use lightspeed_auth::dto::change_password_dto::ChangePasswordDto;
use lightspeed_auth::dto::create_login_dto::CreateLoginDto;
use lightspeed_auth::dto::reset_password_dto::ResetPasswordDto;
use lightspeed_auth::model::auth_account::AuthAccountStatus;
use lightspeed_auth::model::token::TokenType;
use lightspeed_auth::repository::{AuthAccountRepository, AuthRepositoryManager};
use lightspeed_auth::service::auth_account::LsAuthAccountService;
use lightspeed_core::error::{ErrorCodes, ErrorDetail, LsError};
use lightspeed_core::model::language::Language;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};
use lightspeed_test_utils::tokio_test;
use std::collections::HashMap;

#[test]
fn should_create_pending_user() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");
        let password = new_hyphenated_uuid();

        let (user, token) = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await?;

        assert_eq!(username, user.data.username);

        assert!(auth_module.password_codec.verify_match(&password, &user.data.password).await?);

        assert!(user.data.roles.is_empty());

        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);
        assert_eq!(username, token.data.username);

        assert_eq!(TokenType::AccountActivation, token.data.token_type);

        Ok(())
    })
}

#[test]
fn should_assign_default_roles_at_account_creation() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");
        let password = new_hyphenated_uuid();

        let mut auth_config = auth_module.auth_config.clone();
        auth_config.default_roles_on_account_creation = vec![new_hyphenated_uuid()];

        let auth_account_service = LsAuthAccountService::new(
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
                language: Language::En,
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
fn should_return_user_by_id() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, false).await?;

        auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                let user_by_id = auth_module.auth_account_service.fetch_by_user_id_with_conn(conn, user.id).await?;

                assert_eq!(user.data.username, user_by_id.data.username);

                Ok(())
            })
            .await
    })
}

#[test]
fn should_return_user_by_username() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, false).await?;

        auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                let user_by_id =
                    auth_module.auth_account_service.fetch_by_username_with_conn(conn, &user.data.username).await?;

                assert_eq!(user.id, user_by_id.id);

                Ok(())
            })
            .await
    })
}

#[test]
fn should_use_the_email_as_username_if_not_provided() -> Result<(), LsError> {
    tokio_test(async {
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
                language: Language::En,
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
fn should_use_the_email_as_username_if_username_is_empty() -> Result<(), LsError> {
    tokio_test(async {
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
                language: Language::En,
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
fn should_activate_user() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, token) = create_user(auth_module, false).await?;

        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);
        assert_eq!(TokenType::AccountActivation, token.data.token_type);

        let activated_user = auth_module.auth_account_service.activate_user(&token.data.token).await?;

        assert_eq!(AuthAccountStatus::Active, activated_user.data.status);

        assert_eq!(user.data.username, activated_user.data.username);

        assert!(auth_module.auth_account_service.activate_user(&token.data.token).await.is_err());

        assert!(
            auth_module
                .repo_manager
                .c3p0()
                .transaction(async |conn| {
                    auth_module.token_service.fetch_by_token_with_conn(conn, &token.data.token, false).await
                })
                .await
                .is_err()
        );

        Ok(())
    })
}

#[test]
fn should_activate_user_only_if_activation_token_type() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, false).await?;
        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);

        auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                let token = auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
                    .await?;

                let activation_result =
                    auth_module.auth_account_service.activate_user_with_conn(conn, &token.data.token).await;

                assert!(activation_result.is_err());

                Ok(())
            })
            .await
    })
}

#[test]
fn should_activate_user_only_if_pending_activation() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, true).await?;
        assert_eq!(AuthAccountStatus::Active, user.data.status);

        auth_module
            .repo_manager
            .c3p0()
            .transaction::<_, LsError, _>(async |conn| {
                let token = auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::AccountActivation)
                    .await?;

                let activation_result =
                    auth_module.auth_account_service.activate_user_with_conn(conn, &token.data.token).await;

                assert!(activation_result.is_err());

                Ok(())
            })
            .await?;

        Ok(())
    })
}

#[test]
fn should_regenerate_activation_token_by_email_and_username() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let (user, token) = create_user(auth_module, false).await?;

        // use wrong email
        assert!(
            auth_module
                .auth_account_service
                .generate_new_activation_token_by_username_and_email(&user.data.username, "email")
                .await
                .is_err()
        );

        // use wrong username
        assert!(
            auth_module
                .auth_account_service
                .generate_new_activation_token_by_username_and_email("name", &user.data.email)
                .await
                .is_err()
        );

        let (new_user, new_token) = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&user.data.username, &user.data.email)
            .await?;

        assert_eq!(user.id, new_user.id);
        assert!(token.id != new_token.id);
        assert!(token.data.token != new_token.data.token);

        assert!(auth_module.auth_account_service.activate_user(&token.data.token).await.is_err());

        let activated_user = auth_module.auth_account_service.activate_user(&new_token.data.token).await?;

        assert_eq!(AuthAccountStatus::Active, activated_user.data.status);
        assert_eq!(user.id, activated_user.id);

        Ok(())
    })
}

/// Account-enumeration oracle regression test.
///
/// `generate_new_activation_token_by_username_and_email` previously
/// surfaced three distinguishable errors:
///   * `LsError::NotFound` — username unknown,
///   * `LsError::ValidationError` carrying `WRONG_EMAIL` — email mismatch,
///   * `LsError::BadRequest(NOT_PENDING_USER)` — user already active.
///
/// Each told an attacker something different. After the fix, every
/// "request not eligible" branch returns the SAME error variant and code.
/// This test pins that contract.
#[test]
fn generate_new_activation_token_should_use_uniform_error_for_all_failure_modes() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        // 1. Unknown username.
        let unknown_username = format!("nope_{}", new_hyphenated_uuid());
        let unknown = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&unknown_username, "irrelevant@example.com")
            .await;

        // 2. Existing user, wrong email.
        let (pending_user, _) = create_user(auth_module, false).await?;
        let wrong_email = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&pending_user.data.username, "wrong@example.com")
            .await;

        // 3. Existing user with the correct email but wrong status (Active,
        //    not PendingActivation).
        let (active_user, _) = create_user(auth_module, true).await?;
        let wrong_status = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&active_user.data.username, &active_user.data.email)
            .await;

        for (label, result) in
            [("unknown_username", unknown), ("wrong_email", wrong_email), ("wrong_status", wrong_status)]
        {
            match result {
                Err(LsError::BadRequest { code, .. }) => {
                    assert_eq!(
                        ErrorCodes::WRONG_CREDENTIALS,
                        code,
                        "case [{label}] returned a distinguishable error code [{code}]; that's an enumeration oracle"
                    );
                }
                Err(other) => panic!("case [{label}] expected BadRequest(WRONG_CREDENTIALS), got {other:?}"),
                Ok(_) => panic!("case [{label}] expected BadRequest(WRONG_CREDENTIALS), got Ok"),
            }
        }

        Ok(())
    })
}

#[test]
fn should_regenerate_activation_token_by_email_and_username_even_if_token_expired() -> Result<(), LsError> {
    tokio_test(async {
        // Run serially: this test installs an expired token row and then
        // observes its effect across multiple transactions. A concurrent
        // test minting a new token would trigger the lazy sweep and could
        // delete this row mid-test.
        let data = data(true).await;
        let auth_module = &data.0;
        let (user, mut token) = create_user(auth_module, false).await?;
        token.data.expire_at_epoch_seconds = 0;

        let token_model =
            auth_module.repo_manager.c3p0().transaction(async |conn| conn.update(token.clone()).await).await?;

        // The expired token cannot be used to activate or to fetch with validation.
        assert!(
            auth_module
                .repo_manager
                .c3p0()
                .transaction(async |conn| {
                    auth_module.token_service.fetch_by_token_with_conn(conn, &token_model.data.token, true).await
                })
                .await
                .is_err()
        );
        assert!(auth_module.auth_account_service.activate_user(&token_model.data.token).await.is_err());

        // The username+email recovery path must still succeed: knowledge of the
        // matching email is the authentication factor here, and the user must
        // not be locked out just because their previous activation token aged out.
        let (new_user, new_token) = auth_module
            .auth_account_service
            .generate_new_activation_token_by_username_and_email(&user.data.username, &user.data.email)
            .await?;

        assert_eq!(user.id, new_user.id);
        assert!(token_model.data.token != new_token.data.token);

        // The new token activates the user.
        assert!(auth_module.auth_account_service.activate_user(&new_token.data.token).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_login_active_user() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        let auth_validity_seconds = auth_module.auth_config.auth_session_max_validity_minutes * 60;
        let before_login_ts_seconds = current_epoch_seconds();

        let auth = auth_module.auth_account_service.login(&user.data.username, password).await?;

        let after_login_ts_seconds = current_epoch_seconds();

        assert_eq!(user.data.username, auth.username);

        assert!(auth.creation_ts_seconds >= before_login_ts_seconds);
        assert!(auth.creation_ts_seconds <= after_login_ts_seconds);

        assert!(auth.expiration_ts_seconds >= before_login_ts_seconds + auth_validity_seconds as i64);
        assert!(auth.expiration_ts_seconds <= after_login_ts_seconds + auth_validity_seconds as i64);

        Ok(())
    })
}

#[test]
fn should_not_login_inactive_user() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, false).await?;

        let result = auth_module.auth_account_service.login(&user.data.username, password).await;

        match result {
            Err(LsError::BadRequest { code, message }) => {
                assert_eq!(ErrorCodes::INACTIVE_USER, code);
                assert_eq!(format!("User [{}] not in status Active", &user.data.username), message);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_return_wrong_credentials_on_login_of_inactive_user_with_wrong_password() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, false).await?;

        let result = auth_module.auth_account_service.login(&user.data.username, "wrong_password").await;

        match result {
            Err(LsError::BadRequest { code, message }) => {
                assert_eq!(ErrorCodes::WRONG_CREDENTIALS, code);
                assert_eq!(format!("Wrong credentials"), message);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_not_login_with_wrong_username() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&format!("{}_", user.data.username), password).await.is_err());

        Ok(())
    })
}

#[test]
fn should_not_login_with_wrong_password() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, &format!("{password}_")).await.is_err());

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_passwords_do_not_match() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: new_hyphenated_uuid(),
                password_confirm: new_hyphenated_uuid(),
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("password"))
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_password_too_short() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");
        let min_len = auth_module.auth_config.min_password_len;
        let password = "a".repeat(min_len - 1);

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await;

        match result {
            Err(LsError::ValidationError { details }) => {
                let errors = details.details.get("password").expect("expected password error");
                let expected = ErrorDetail::new(ERR_PASSWORD_TOO_SHORT, vec![min_len.to_string()]);
                assert!(errors.contains(&expected));
            }
            _ => panic!("expected ValidationError"),
        }

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_not_valid_email() -> Result<(), LsError> {
    tokio_test(async {
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
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("email"))
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_not_accepted_privacy_policy() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");
        let password = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: false,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await;

        assert!(result.is_err());

        match &result {
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("accept_privacy_policy"))
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_username_not_unique() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();

        let mut dto = CreateLoginDto {
            username: Some(new_hyphenated_uuid()),
            email: format!("{}@email.fake", new_hyphenated_uuid()),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::En,
            password: password.clone(),
            password_confirm: password.clone(),
        };

        assert!(auth_module.auth_account_service.create_user(dto.clone()).await.is_ok());

        dto.email = format!("{}@email.fake", new_hyphenated_uuid());

        let result = auth_module.auth_account_service.create_user(dto).await;
        assert!(result.is_err());

        match &result {
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("username"))
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn create_user_should_fail_if_email_not_unique() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();

        let mut dto = CreateLoginDto {
            username: Some(new_hyphenated_uuid()),
            email: format!("{}@email.fake", new_hyphenated_uuid()),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::En,
            password: password.clone(),
            password_confirm: password.clone(),
        };

        assert!(auth_module.auth_account_service.create_user(dto.clone()).await.is_ok());

        dto.username = Some(new_hyphenated_uuid());
        let result = auth_module.auth_account_service.create_user(dto).await;
        assert!(result.is_err());

        match &result {
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("email"))
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_reset_password_by_token() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = &create_user_with_password(auth_module, &password, true).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
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
fn should_fail_resetting_password_if_token_expired() -> Result<(), LsError> {
    tokio_test(async {
        // Run serially: a concurrent test minting a new token would trigger
        // the lazy sweep and delete this test's expired row before the reset
        // path's fetch_by_token reads it, turning the expected expiry
        // ValidationError into a NotFound.
        let data = data(true).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        let mut token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
                    .await
            })
            .await?;

        token.data.expire_at_epoch_seconds = 0;
        let token = auth_module.repo_manager.c3p0().transaction(async |conn| conn.update(token.clone()).await).await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: password_new.clone(),
                token: token.data.token,
                password_confirm: password_new.clone(),
            })
            .await;

        match result {
            Err(LsError::ValidationError { details }) => {
                assert_eq!("expired", details.details["expire_at_epoch"][0]);
            }
            _ => panic!("expected ValidationError for expired reset token"),
        }

        // Original password must still work — the reset must not have taken effect.
        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_ok());
        assert!(auth_module.auth_account_service.login(&user.data.username, &password_new).await.is_err());

        Ok(())
    })
}

#[test]
fn should_reset_password_only_if_correct_token_type() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::AccountActivation)
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
fn should_reset_password_only_if_user_is_active() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, false).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
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
fn should_reset_password_only_if_passwords_match() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, false).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
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
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("password"))
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn should_fail_resetting_password_if_password_too_short() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
                    .await
            })
            .await?;

        let min_len = auth_module.auth_config.min_password_len;
        let short_password = "a".repeat(min_len - 1);

        let result = auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                password: short_password.clone(),
                token: token.data.token,
                password_confirm: short_password.clone(),
            })
            .await;

        match result {
            Err(LsError::ValidationError { details }) => {
                let errors = details.details.get("password").expect("expected password error");
                let expected = ErrorDetail::new(ERR_PASSWORD_TOO_SHORT, vec![min_len.to_string()]);
                assert!(errors.contains(&expected));
            }
            _ => panic!("expected ValidationError"),
        }

        // Original password must still work.
        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_generate_reset_password_token() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, true).await?;

        let (new_user, token) =
            auth_module.auth_account_service.generate_reset_password_token(&user.data.username).await?;
        assert_eq!(user.id, new_user.id);
        assert_eq!(TokenType::ResetPassword, token.data.token_type);
        Ok(())
    })
}

#[test]
fn should_not_generate_reset_password_token_if_user_not_active() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let (user, _) = create_user(auth_module, false).await?;

        assert!(auth_module.auth_account_service.generate_reset_password_token(&user.data.username).await.is_err());

        Ok(())
    })
}

#[test]
fn should_change_user_password() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

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
fn should_not_change_user_password_if_wrong_old_password() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        let password_new = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: format!("__{password}__"),
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
fn should_not_change_user_password_if_inactive_user() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, false).await?;

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
fn should_not_change_user_password_if_new_passwords_do_not_match() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

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
            Err(LsError::ValidationError { details }) => {
                assert!(details.details.contains_key("new_password"))
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_not_change_user_password_if_new_password_too_short() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        let min_len = auth_module.auth_config.min_password_len;
        let short_password = "a".repeat(min_len - 1);

        let result = auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: password.clone(),
                new_password: short_password.clone(),
                new_password_confirm: short_password.clone(),
            })
            .await;

        match result {
            Err(LsError::ValidationError { details }) => {
                let errors = details.details.get("new_password").expect("expected new_password error");
                let expected = ErrorDetail::new(ERR_PASSWORD_TOO_SHORT, vec![min_len.to_string()]);
                assert!(errors.contains(&expected));
            }
            _ => panic!("expected ValidationError"),
        }

        // Original password must still work.
        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_add_and_remove_roles() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let auh_service = &auth_module.auth_account_service;

        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");
        let password = new_hyphenated_uuid();

        let (user, _) = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })
            .await?;

        assert!(user.data.roles.is_empty());

        let user = auh_service.add_roles(user.id, &[]).await?;
        assert!(user.data.roles.is_empty());

        let user = auh_service.delete_roles(user.id, &["one".to_owned()]).await?;
        assert!(user.data.roles.is_empty());

        let user = auh_service.add_roles(user.id, &["one".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned()], user.data.roles);

        let user = auh_service.add_roles(user.id, &["two".to_owned(), "three".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned(), "two".to_owned(), "three".to_owned()], user.data.roles);

        let user = auh_service.delete_roles(user.id, &["two".to_owned(), "four".to_owned()]).await?;
        assert_eq!(vec!["one".to_owned(), "three".to_owned()], user.data.roles);

        let user = auh_service.delete_roles(user.id, &["one".to_owned(), "three".to_owned()]).await?;
        assert!(user.data.roles.is_empty());

        Ok(())
    })
}

#[test]
fn should_change_username() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

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
fn should_change_email() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

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
fn should_change_username_and_email() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let new_username = new_hyphenated_uuid();
        let new_email = format!("{new_username}@test.com");
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
fn should_disable_an_active_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let updated_user = auth_module.auth_account_service.disable_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(AuthAccountStatus::Disabled, updated_user.data.status);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        let loaded_user = auth_module.auth_account_service.fetch_by_user_id(user.id).await.unwrap();

        assert_eq!(AuthAccountStatus::Disabled, loaded_user.data.status);

        Ok(())
    })
}

#[test]
fn should_fail_disabling_a_pending_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, false).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        // Act
        let result = auth_module.auth_account_service.disable_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_fail_disabling_a_disabled_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

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
fn should_activate_a_disabled_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.disable_by_user_id(user.id).await.is_ok());

        // Act
        let updated_user = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(AuthAccountStatus::Active, updated_user.data.status);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        let loaded_user = auth_module.auth_account_service.fetch_by_user_id(user.id).await.unwrap();

        assert_eq!(AuthAccountStatus::Active, loaded_user.data.status);

        Ok(())
    })
}

#[test]
fn should_fail_reactivating_a_pending_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, false).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        // Act
        let result = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_fail_reactivating_an_active_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_ok());

        // Act
        let result = auth_module.auth_account_service.reactivate_disabled_user_by_user_id(user.id).await;

        // Assert
        assert!(result.is_err());

        Ok(())
    })
}

#[test]
fn should_delete_a_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        // Act
        let deleted_user_count = auth_module.auth_account_service.delete_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(1, deleted_user_count);

        assert!(auth_module.auth_account_service.login(&user.data.username, password).await.is_err());

        assert!(auth_module.auth_account_service.fetch_by_user_id(user.id).await.is_err());

        Ok(())
    })
}

#[test]
fn should_not_fail_deleting_a_deleted_user() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;
        let password = "123456789";
        let (user, _) = create_user_with_password(auth_module, password, true).await?;

        auth_module.auth_account_service.delete_by_user_id(user.id).await.unwrap();

        // Act
        let result = auth_module.auth_account_service.delete_by_user_id(user.id).await.unwrap();

        // Assert
        assert_eq!(0, result);

        Ok(())
    })
}

#[test]
fn should_return_users_by_status() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(false).await;
        let auth_module = &data.0;

        let (user_active_1, _) = create_user(auth_module, true).await?;
        let (user_active_2, _) = create_user(auth_module, true).await?;
        let (user_pending_1, _) = create_user(auth_module, false).await?;
        let (user_pending_2, _) = create_user(auth_module, false).await?;
        let (user_disabled_1, _) = create_user(auth_module, true).await?;

        assert!(auth_module.auth_account_service.disable_by_user_id(user_disabled_1.id).await.is_ok());

        // Act
        let all_active_users =
            auth_module.auth_account_service.fetch_all_by_status(AuthAccountStatus::Active, 0, u32::MAX).await.unwrap();

        let all_pending_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::PendingActivation, 0, u32::MAX)
            .await
            .unwrap();

        let all_disabled_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::Disabled, 0, u32::MAX)
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
fn should_return_users_by_status_with_offset_and_limit() -> Result<(), LsError> {
    tokio_test(async {
        // Arrange
        let data = data(true).await;
        let auth_module = &data.0;

        let (user_1, _) = create_user(auth_module, true).await?;
        let (user_2, _) = create_user(auth_module, true).await?;
        let (_user_3, _) = create_user(auth_module, true).await?;

        // Act
        let all_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::Active, user_1.id, u32::MAX)
            .await
            .unwrap();

        let offset_one_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::Active, user_2.id, u32::MAX)
            .await
            .unwrap();

        let limit_two_users = auth_module
            .auth_account_service
            .fetch_all_by_status(AuthAccountStatus::Active, user_2.id, 2)
            .await
            .unwrap();

        // Assert
        assert_eq!(all_users[1].id, offset_one_users[0].id);
        assert_eq!(all_users.len() - 1, offset_one_users.len());
        assert_eq!(all_users[1].id, limit_two_users[0].id);
        assert_eq!(2, limit_two_users.len());

        Ok(())
    })
}

/// Build an `LsAuthAccountService` that shares state with `auth_module` but
/// uses a customized `AuthConfig`. Used by tests that need to flip
/// `password_expiration_seconds` without rebuilding the whole module.
fn auth_account_service_with_config<RepoManager: AuthRepositoryManager>(
    auth_module: &lightspeed_auth::LsAuthModule<RepoManager>,
    auth_config: AuthConfig,
) -> LsAuthAccountService<RepoManager> {
    LsAuthAccountService::new(
        auth_module.repo_manager.c3p0().clone(),
        auth_config,
        auth_module.token_service.clone(),
        auth_module.password_codec.clone(),
        auth_module.repo_manager.auth_account_repo(),
    )
}

/// Overwrite a user's stored timestamps by going through the repo. Tests use
/// this to simulate an aged password without actually waiting.
async fn set_user_timestamps<RepoManager: AuthRepositoryManager>(
    auth_module: &lightspeed_auth::LsAuthModule<RepoManager>,
    user_id: i64,
    password_updated: i64,
    created: Option<i64>,
) -> Result<(), LsError> {
    let repo = auth_module.repo_manager.auth_account_repo();
    auth_module
        .repo_manager
        .c3p0()
        .transaction(async |conn| {
            let mut row = repo.fetch_by_id(conn, user_id).await?;
            row.data.password_updated_date_epoch_seconds = password_updated;
            if let Some(created) = created {
                row.data.created_date_epoch_seconds = created;
            }
            repo.update(conn, row).await?;
            Ok::<_, LsError>(())
        })
        .await
}

#[test]
fn should_honor_configured_min_password_len() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        // Build a service that requires a longer minimum password length than the default.
        let mut auth_config = auth_module.auth_config.clone();
        auth_config.min_password_len = 16;

        let auth_account_service = LsAuthAccountService::new(
            auth_module.repo_manager.c3p0().clone(),
            auth_config.clone(),
            auth_module.token_service.clone(),
            auth_module.password_codec.clone(),
            auth_module.repo_manager.auth_account_repo(),
        );

        let username = new_hyphenated_uuid();
        let email = format!("{username}@email.fake");

        // 15-byte password is rejected because the configured limit is 16.
        let short_password = "a".repeat(auth_config.min_password_len - 1);
        let result = auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: short_password.clone(),
                password_confirm: short_password.clone(),
            })
            .await;

        match result {
            Err(LsError::ValidationError { details }) => {
                let errors = details.details.get("password").expect("expected password error");
                let expected = ErrorDetail::new(ERR_PASSWORD_TOO_SHORT, vec![auth_config.min_password_len.to_string()]);
                assert!(errors.contains(&expected));
            }
            _ => panic!("expected ValidationError"),
        }

        // A password that meets the configured limit succeeds.
        let ok_password = "a".repeat(auth_config.min_password_len);
        let (user, _) = auth_account_service
            .create_user(CreateLoginDto {
                username: Some(username.clone()),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: ok_password.clone(),
                password_confirm: ok_password.clone(),
            })
            .await?;
        assert_eq!(username, user.data.username);

        Ok(())
    })
}

#[test]
fn should_fail_login_when_password_expired() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        // Backdate the password timestamp by 1 hour.
        let backdated_to = current_epoch_seconds() - 3600;
        set_user_timestamps(auth_module, user.id, backdated_to, None).await?;

        // Service that considers passwords older than 60s expired.
        let mut auth_config = auth_module.auth_config.clone();
        auth_config.password_expiration_seconds = Some(60);
        let service = auth_account_service_with_config(auth_module, auth_config);

        let result = service.login(&user.data.username, &password).await;

        match result {
            Err(LsError::BadRequest { code, .. }) => {
                assert_eq!(ErrorCodes::EXPIRED_PASSWORD, code);
            }
            _ => panic!("expected BadRequest with EXPIRED_PASSWORD"),
        }

        // The default service (expiration disabled) still accepts the login.
        assert!(auth_module.auth_account_service.login(&user.data.username, &password).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_login_when_password_within_expiration_window() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        // 1 hour expiration; the user was just created, so well within window.
        let mut auth_config = auth_module.auth_config.clone();
        auth_config.password_expiration_seconds = Some(3600);
        let service = auth_account_service_with_config(auth_module, auth_config);

        assert!(service.login(&user.data.username, &password).await.is_ok());

        Ok(())
    })
}

#[test]
fn change_password_should_reset_password_age() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        // Make the password "old".
        let backdated_to = current_epoch_seconds() - 3600;
        set_user_timestamps(auth_module, user.id, backdated_to, None).await?;

        // Sanity-check: with a 60s expiration the login is rejected before changing.
        let mut auth_config = auth_module.auth_config.clone();
        auth_config.password_expiration_seconds = Some(60);
        let strict_service = auth_account_service_with_config(auth_module, auth_config.clone());
        match strict_service.login(&user.data.username, &password).await {
            Err(LsError::BadRequest { code, .. }) => assert_eq!(ErrorCodes::EXPIRED_PASSWORD, code),
            _ => panic!("expected EXPIRED_PASSWORD before change_password"),
        }

        // Change the password through the default service.
        let new_password = new_hyphenated_uuid();
        auth_module
            .auth_account_service
            .change_password(ChangePasswordDto {
                user_id: user.id,
                old_password: password.clone(),
                new_password: new_password.clone(),
                new_password_confirm: new_password.clone(),
            })
            .await?;

        // Login with the new password against the strict service must now succeed.
        assert!(strict_service.login(&user.data.username, &new_password).await.is_ok());

        Ok(())
    })
}

#[test]
fn reset_password_should_reset_password_age() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(auth_module, &password, true).await?;

        // Make the password "old".
        let backdated_to = current_epoch_seconds() - 3600;
        set_user_timestamps(auth_module, user.id, backdated_to, None).await?;

        // Issue a reset-password token and use it to set a new password.
        let token = auth_module
            .repo_manager
            .c3p0()
            .transaction(async |conn| {
                auth_module
                    .token_service
                    .generate_and_save_token_with_conn(conn, &user.data.username, TokenType::ResetPassword)
                    .await
            })
            .await?;

        let new_password = new_hyphenated_uuid();
        auth_module
            .auth_account_service
            .reset_password_by_token(ResetPasswordDto {
                token: token.data.token,
                password: new_password.clone(),
                password_confirm: new_password.clone(),
            })
            .await?;

        let mut auth_config = auth_module.auth_config.clone();
        auth_config.password_expiration_seconds = Some(60);
        let strict_service = auth_account_service_with_config(auth_module, auth_config);

        assert!(strict_service.login(&user.data.username, &new_password).await.is_ok());

        Ok(())
    })
}
