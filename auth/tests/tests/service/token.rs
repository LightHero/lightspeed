use crate::test;
use c3p0::NewModel;
use lightspeed_auth::model::token::{TokenType, TokenData};
use lightspeed_core::utils::{new_hyphenated_uuid, current_epoch_seconds};

#[test]
fn should_delete_token() {
    test(|auth_module| {
        let c3p0 = &auth_module.c3p0;

        let token = NewModel {
            version: 0,
            data: TokenData {
                token: "123454678".to_owned(),
                expire_at_epoch: 9999999999999,
                token_type: TokenType::ResetPassword,
                username: "test@test.com".to_owned(),
            },
        };

        let saved_token = auth_module.token_repo.save(&c3p0.connection()?, token)?;

        assert!(auth_module
            .token_repo
            .exists_by_id(&c3p0.connection()?, &saved_token.id)?);
        assert_eq!(1, auth_module.token_service.delete(&c3p0.connection()?, saved_token.clone())?);
        assert!(!auth_module
            .token_repo
            .exists_by_id(&c3p0.connection()?, &saved_token.id)?);

        Ok(())
    });
}

#[test]
fn should_generate_token() {
    test(|auth_module| {
        let c3p0 = &auth_module.c3p0;
        Ok(c3p0.transaction(|conn| {
            let username = new_hyphenated_uuid();
            let token_type = TokenType::AccountActivation;

            let before = current_epoch_seconds();
            let token = auth_module.token_service.generate_and_save_token(conn, username.clone(), token_type.clone())?;
            let after = current_epoch_seconds();

            let expiration_seconds = &auth_module.auth_config.token_activation_validity_minutes * 60;

            assert_eq!(username, token.data.username);

            match token.data.token_type {
                TokenType::AccountActivation => {},
                _ => assert!(false)
            }
            assert!((before + expiration_seconds) <= token.data.expire_at_epoch);
            assert!((after + expiration_seconds) >= token.data.expire_at_epoch);

            assert!(auth_module.token_service.fetch_by_token(conn, &token.data.token, true).unwrap().is_some());
            assert_eq!(1, auth_module.token_service.delete(conn, token.clone())?);
            assert!(auth_module.token_service.fetch_by_token(conn, &token.data.token, true).unwrap().is_none());

            Ok(())
        })?)
    });
}

#[test]
fn should_validate_token_on_fetch() {
    test(|auth_module| {
        let c3p0 = &auth_module.c3p0;
        Ok(c3p0.transaction(|conn| {

            let token = NewModel {
                version: 0,
                data: TokenData {
                    token: "123454678".to_owned(),
                    expire_at_epoch: current_epoch_seconds() - 1,
                    token_type: TokenType::ResetPassword,
                    username: "test@test.com".to_owned(),
                },
            };

            let saved_token = auth_module.token_repo.save(&c3p0.connection()?, token)?;

            assert!(auth_module.token_service.fetch_by_token(conn, &saved_token.data.token, true).is_err());

            Ok(())
        })?)
    });
}