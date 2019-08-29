use crate::test;
use c3p0::*;
use lightspeed_auth::model::token::{TokenData, TokenType};
use lightspeed_auth::repository::AuthRepositoryManager;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};

#[test]
fn should_delete_token() {
    test(|auth_module, _| {
        let c3p0 = auth_module.repo_manager.c3p0();
        let token_repo = auth_module.repo_manager.token_repo();

        let token = NewModel {
            version: 0,
            data: TokenData {
                token: new_hyphenated_uuid(),
                expire_at_epoch: 9999999999999,
                token_type: TokenType::ResetPassword,
                username: "test@test.com".to_owned(),
            },
        };

        let saved_token = token_repo.save(&c3p0.connection()?, token)?;

        assert!(token_repo.exists_by_id(&c3p0.connection()?, &saved_token.id)?);
        assert_eq!(
            1,
            auth_module
                .token_service
                .delete(&c3p0.connection()?, saved_token.clone())?
        );
        assert!(!token_repo.exists_by_id(&c3p0.connection()?, &saved_token.id)?);

        Ok(())
    });
}

#[test]
fn should_generate_token() {
    test(|auth_module, _| {
        let c3p0 = auth_module.repo_manager.c3p0();
        c3p0.transaction(|conn| {
            let username = new_hyphenated_uuid();
            let token_type = TokenType::AccountActivation;

            let before = current_epoch_seconds();
            let token = auth_module.token_service.generate_and_save_token(
                conn,
                username.clone(),
                token_type.clone(),
            )?;
            let after = current_epoch_seconds();

            let expiration_seconds =
                &auth_module.auth_config.token_activation_validity_minutes * 60;

            assert_eq!(username, token.data.username);

            match token.data.token_type {
                TokenType::AccountActivation => {}
                _ => assert!(false),
            }
            assert!((before + expiration_seconds) <= token.data.expire_at_epoch);
            assert!((after + expiration_seconds) >= token.data.expire_at_epoch);

            assert!(auth_module
                .token_service
                .fetch_by_token(conn, &token.data.token, true)
                .is_ok());
            assert_eq!(1, auth_module.token_service.delete(conn, token.clone())?);
            assert!(auth_module
                .token_service
                .fetch_by_token(conn, &token.data.token, true)
                .is_err());
            Ok(())
        })
    });
}

#[test]
fn should_validate_token_on_fetch() {
    test(|auth_module, _| {
        let c3p0 = auth_module.repo_manager.c3p0();
        let token_repo = auth_module.repo_manager.token_repo();

        c3p0.transaction(|conn| {
            let token = NewModel {
                version: 0,
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch: current_epoch_seconds() - 1,
                    token_type: TokenType::ResetPassword,
                    username: "test@test.com".to_owned(),
                },
            };

            let saved_token = token_repo.save(conn, token)?;

            assert!(auth_module
                .token_service
                .fetch_by_token(conn, &saved_token.data.token, true)
                .is_err());

            Ok(())
        })
    });
}
