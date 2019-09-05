use crate::test;
use crate::tests::util::{create_user, create_user_with_password, filter_emails_to};
use c3p0::*;
use lightspeed_auth::dto::change_password_dto::ChangePasswordDto;
use lightspeed_auth::dto::create_user_dto::CreateLoginDto;
use lightspeed_auth::dto::reset_password_dto::ResetPasswordDto;
use lightspeed_auth::model::auth_account::AuthAccountStatus;
use lightspeed_auth::model::token::TokenType;
use lightspeed_auth::repository::AuthRepositoryManager;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::model::language::Language;
use lightspeed_core::utils::new_hyphenated_uuid;
use std::collections::HashMap;

#[test]
fn should_create_pending_user() {
    test(|auth_module, _| {
        let username = new_hyphenated_uuid();
        let email = format!("{}@email.fake", username);
        let password = new_hyphenated_uuid();

        let (user, token) = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: username.clone(),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            })?;

        assert_eq!(username, user.data.username);

        assert!(auth_module
            .password_codec
            .verify_match(&password, &user.data.password)?);

        assert!(user.data.roles.is_empty());

        /*
            assertFalse(user.data.roles.contains(Roles.MARKET_BID))
            assertFalse(user.data.roles.contains(Roles.MARKET_BUY))
            assertFalse(user.data.roles.contains(Roles.MARKET_ASK))
            assertFalse(user.data.roles.contains(Roles.UM_VALID))
        assert!(user.data.roles.contains(Roles.PROFILE_PRIVATE))
        */

        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);
        assert_eq!(username, token.data.username);

        assert_eq!(TokenType::AccountActivation, token.data.token_type);

        Ok(())
    });
}

#[test]
fn should_send_activation_email() {
    test(|auth_module, email_module| {
        let (user, token) = create_user(&auth_module, false)?;

        let emails = filter_emails_to(&user.data.email, email_module.email_service.get_emails()?);
        assert_eq!(1, emails.len());

        let public_token_url = auth_module.token_service.generate_public_token_url(&token);
        assert!(emails[0].html.clone().unwrap().contains(&public_token_url));

        Ok(())
    });
}

#[test]
fn should_return_user_by_id() {
    test(|auth_module, _| {
        let (user, _) = create_user(&auth_module, false)?;

        let conn = &auth_module.repo_manager.c3p0().connection()?;
        let user_by_id = auth_module
            .auth_account_service
            .fetch_by_user_id(conn, user.id)?;

        assert_eq!(user.data.username, user_by_id.data.username);
        Ok(())
    });
}

#[test]
fn should_return_user_by_username() {
    test(|auth_module, _| {
        let (user, _) = create_user(&auth_module, false)?;

        let conn = &auth_module.repo_manager.c3p0().connection()?;
        let user_by_id = auth_module
            .auth_account_service
            .fetch_by_username(conn, &user.data.username)?;

        assert_eq!(user.id, user_by_id.id);
        Ok(())
    });
}

#[test]
fn should_activate_user() {
    test(|auth_module, _| {
        let (user, token) = create_user(&auth_module, false)?;

        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);
        assert_eq!(TokenType::AccountActivation, token.data.token_type);

        let activated_user = auth_module
            .auth_account_service
            .activate_user(&token.data.token)?;

        assert_eq!(AuthAccountStatus::Active, activated_user.data.status);

        assert_eq!(user.data.username, activated_user.data.username);

        assert!(auth_module
            .auth_account_service
            .activate_user(&token.data.token)
            .is_err());

        let conn = &auth_module.repo_manager.c3p0().connection()?;

        assert!(auth_module
            .token_service
            .fetch_by_token(conn, &token.data.token, false)
            .is_err());

        Ok(())
    });
}

#[test]
fn should_activate_user_only_if_activation_token_type() {
    test(|auth_module, _| {
        let (user, _) = create_user(&auth_module, false)?;
        assert_eq!(AuthAccountStatus::PendingActivation, user.data.status);

        auth_module.repo_manager.c3p0().transaction(|conn| {
            let token = auth_module.token_service.generate_and_save_token(
                conn,
                &user.data.username,
                TokenType::ResetPassword,
            )?;

            let activation_result = auth_module
                .auth_account_service
                .activate_user(&token.data.token);

            assert!(activation_result.is_err());

            Ok(())
        })
    });
}

#[test]
fn should_activate_user_only_if_pending_activation() {
    test(|auth_module, _| {
        let (user, _) = create_user(&auth_module, true)?;
        assert_eq!(AuthAccountStatus::Active, user.data.status);

        auth_module.repo_manager.c3p0().transaction(|conn| {
            let token = auth_module.token_service.generate_and_save_token(
                conn,
                &user.data.username,
                TokenType::AccountActivation,
            )?;

            let activation_result = auth_module
                .auth_account_service
                .activate_user(&token.data.token);

            assert!(activation_result.is_err());

            Ok(())
        })
    });
}

#[test]
fn should_resend_activation_token_email() {
    test(|auth_module, email_module| {
        let (user, token) = create_user(&auth_module, false)?;
        //email_module.email_service.clear_emails()?;
        let emails_len_before =
            filter_emails_to(&user.data.email, email_module.email_service.get_emails()?).len();

        let (new_user, new_token) = auth_module
            .auth_account_service
            .send_new_activation_token_by_email(&token.data.token)?;
        assert_eq!(user.id, new_user.id);
        assert!(!(token.id == new_token.id));
        assert!(!(token.data.token == new_token.data.token));

        let emails = filter_emails_to(&user.data.email, email_module.email_service.get_emails()?);
        assert_eq!(emails_len_before + 1, emails.len());
        assert!(!emails[emails_len_before]
            .html
            .clone()
            .unwrap()
            .contains(&token.data.token));
        assert!(emails[emails_len_before]
            .html
            .clone()
            .unwrap()
            .contains(&new_token.data.token));

        assert!(auth_module
            .auth_account_service
            .activate_user(&token.data.token)
            .is_err());

        let activated_user = auth_module
            .auth_account_service
            .activate_user(&new_token.data.token)?;

        assert_eq!(AuthAccountStatus::Active, activated_user.data.status);
        assert_eq!(user.id, activated_user.id);

        Ok(())
    });
}

#[test]
fn should_resend_activation_token_even_if_token_expired() {
    test(|auth_module, _| {
        let (_, mut token) = create_user(&auth_module, false)?;
        token.data.expire_at_epoch = 0;
        token = auth_module
            .repo_manager
            .token_repo()
            .update(&auth_module.repo_manager.c3p0().connection()?, token)?;
        assert!(auth_module
            .token_service
            .fetch_by_token(
                &auth_module.repo_manager.c3p0().connection()?,
                &token.data.token,
                true
            )
            .is_err());

        assert!(auth_module
            .auth_account_service
            .activate_user(&token.data.token)
            .is_err());

        let (_, new_token) = auth_module
            .auth_account_service
            .send_new_activation_token_by_email(&token.data.token)?;

        assert!(auth_module
            .auth_account_service
            .activate_user(&new_token.data.token)
            .is_ok());

        Ok(())
    });
}

#[test]
fn should_resend_activation_token_only_if_correct_token_type() {
    test(|auth_module, _| {
        let (user, _) = create_user(&auth_module, false)?;

        let token = auth_module.token_service.generate_and_save_token(
            &auth_module.repo_manager.c3p0().connection()?,
            &user.data.username,
            TokenType::ResetPassword,
        )?;

        assert!(auth_module
            .auth_account_service
            .send_new_activation_token_by_email(&token.data.token)
            .is_err());

        Ok(())
    });
}

#[test]
fn should_login_active_user() {
    test(|auth_module, _| {
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true)?;

        let auth = auth_module
            .auth_account_service
            .login(&user.data.username, password)?;
        assert_eq!(user.data.username, auth.username);
        Ok(())
    });
}

#[test]
fn should_not_login_inactive_user() {
    test(|auth_module, _| {
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, false)?;

        assert!(auth_module
            .auth_account_service
            .login(&user.data.username, password)
            .is_err());

        Ok(())
    });
}

#[test]
fn should_not_login_with_wrong_username() {
    test(|auth_module, _| {
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true)?;

        assert!(auth_module
            .auth_account_service
            .login(&format!("{}_", user.data.username), password)
            .is_err());

        Ok(())
    });
}

#[test]
fn should_not_login_with_wrong_password() {
    test(|auth_module, _| {
        let password = "123456789";
        let (user, _) = create_user_with_password(&auth_module, password, true)?;

        assert!(auth_module
            .auth_account_service
            .login(&user.data.username, &format!("{}_", password))
            .is_err());

        Ok(())
    });
}

#[test]
fn create_user_should_fail_if_passwords_do_not_match() {
    test(|auth_module, _| {
        let username = new_hyphenated_uuid();
        let email = format!("{}@email.fake", username);

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: username.clone(),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
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
    });
}

#[test]
fn create_user_should_fail_if_not_valid_email() {
    test(|auth_module, _| {
        let username = new_hyphenated_uuid();
        let email = new_hyphenated_uuid();
        let password = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: username.clone(),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: true,
                language: Language::En,
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
    });
}

#[test]
fn create_user_should_fail_if_not_accepted_privacy_policy() {
    test(|auth_module, _| {
        let username = new_hyphenated_uuid();
        let email = format!("{}@email.fake", username);
        let password = new_hyphenated_uuid();

        let result = auth_module
            .auth_account_service
            .create_user(CreateLoginDto {
                username: username.clone(),
                email: email.clone(),
                data: HashMap::new(),
                accept_privacy_policy: false,
                language: Language::En,
                password: password.clone(),
                password_confirm: password.clone(),
            });

        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("accept_privacy_policy"))
            }
            _ => assert!(false),
        }

        Ok(())
    });
}

#[test]
fn create_user_should_fail_if_username_not_unique() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();

        let mut dto = CreateLoginDto {
            username: new_hyphenated_uuid(),
            email: format!("{}@email.fake", new_hyphenated_uuid()),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::En,
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
        }

        Ok(())
    });
}

#[test]
fn create_user_should_fail_if_email_not_unique() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();

        let mut dto = CreateLoginDto {
            username: new_hyphenated_uuid(),
            email: format!("{}@email.fake", new_hyphenated_uuid()),
            data: HashMap::new(),
            accept_privacy_policy: true,
            language: Language::En,
            password: password.clone(),
            password_confirm: password.clone(),
        };

        assert!(auth_module
            .auth_account_service
            .create_user(dto.clone())
            .is_ok());

        dto.username = new_hyphenated_uuid();
        let result = auth_module.auth_account_service.create_user(dto);
        assert!(result.is_err());

        match &result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert!(details.details.contains_key("email"))
            }
            _ => assert!(false),
        }

        Ok(())
    });
}

#[test]
fn should_reset_password_by_token() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true)?;

        let token = auth_module.token_service.generate_and_save_token(
            &auth_module.repo_manager.c3p0().connection()?,
            &user.data.username,
            TokenType::ResetPassword,
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
    });
}

#[test]
fn should_reset_password_only_if_correct_token_type() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, true)?;

        let token = auth_module.token_service.generate_and_save_token(
            &auth_module.repo_manager.c3p0().connection()?,
            &user.data.username,
            TokenType::AccountActivation,
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
    });
}

#[test]
fn should_reset_password_only_if_user_is_active() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, false)?;

        let token = auth_module.token_service.generate_and_save_token(
            &auth_module.repo_manager.c3p0().connection()?,
            &user.data.username,
            TokenType::ResetPassword,
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
    });
}

#[test]
fn should_reset_password_only_if_passwords_match() {
    test(|auth_module, _| {
        let password = new_hyphenated_uuid();
        let (user, _) = create_user_with_password(&auth_module, &password, false)?;

        let token = auth_module.token_service.generate_and_save_token(
            &auth_module.repo_manager.c3p0().connection()?,
            &user.data.username,
            TokenType::ResetPassword,
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
        }

        Ok(())
    });
}

#[test]
fn should_send_reset_password_email() {
    test(|auth_module, email_module| {
        let (user, _) = create_user(&auth_module, true)?;
        let emails_len_before =
            filter_emails_to(&user.data.email, email_module.email_service.get_emails()?).len();

        let (new_user, token) = auth_module
            .auth_account_service
            .send_reset_password_email(&user.data.username)?;
        assert_eq!(user.id, new_user.id);
        assert_eq!(TokenType::ResetPassword, token.data.token_type);

        let emails = filter_emails_to(&user.data.email, email_module.email_service.get_emails()?);
        assert_eq!(emails_len_before + 1, emails.len());

        let public_token_url = auth_module.token_service.generate_public_token_url(&token);
        assert!(emails[emails_len_before]
            .html
            .clone()
            .unwrap()
            .contains(&public_token_url));

        Ok(())
    });
}

#[test]
fn should_not_send_reset_password_email_if_user_not_active() {
    test(|auth_module, email_module| {
        let (user, _) = create_user(&auth_module, false)?;
        let emails_len_before =
            filter_emails_to(&user.data.email, email_module.email_service.get_emails()?).len();

        assert!(auth_module
            .auth_account_service
            .send_reset_password_email(&user.data.username)
            .is_err());

        let emails = filter_emails_to(&user.data.email, email_module.email_service.get_emails()?);
        assert_eq!(emails_len_before, emails.len());

        Ok(())
    });
}

#[test]
fn should_change_user_password() {
    test(|auth_module, _| {
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
    });
}

#[test]
fn should_not_change_user_password_if_wrong_old_password() {
    test(|auth_module, _| {
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
    });
}

#[test]
fn should_not_change_user_password_if_inactive_user() {
    test(|auth_module, _| {
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
    });
}

#[test]
fn should_not_change_user_password_if_new_passwords_do_not_match() {
    test(|auth_module, _| {
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
    });
}

// Continue from shouldResetUserPasswordByToken()
