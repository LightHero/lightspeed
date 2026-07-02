use crate::data;
use c3p0::*;
use lightspeed_account_management::error::LsAccountManagementError;
use lightspeed_account_management::model::token::{TokenData, TokenType};
use lightspeed_account_management::repository::AMRepositoryManager;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};
use maybe_once::tokio_shared;

#[tokio_shared::test]
async fn should_delete_token() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;

    let c3p0 = auth_module.repo_manager.c3p0();

    let token = NewRecord {
        data: TokenData {
            token: new_hyphenated_uuid(),
            expire_at_epoch_seconds: 9999999999999,
            token_type: TokenType::ResetPassword,
            username: "test@test.com".to_owned(),
        },
    };

    c3p0.transaction(async |conn| {
        let saved_token = conn.save(token).await?;

        assert!(conn.exists_by_id::<TokenData>(saved_token.id).await?);
        assert!(auth_module.token_service.delete_with_conn(conn, saved_token.clone()).await.is_ok());
        assert!(!conn.exists_by_id::<TokenData>(saved_token.id).await?);

        Ok(())
    })
    .await
}

#[tokio_shared::test]
async fn should_generate_token() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;

    let c3p0 = auth_module.repo_manager.c3p0();
    c3p0.transaction(async |conn| {
        let username = new_hyphenated_uuid();
        let token_type = TokenType::AccountActivation;

        let before = current_epoch_seconds();
        let token = auth_module
            .token_service
            .generate_and_save_token_with_conn(conn, username.clone(), token_type.clone())
            .await?;
        let after = current_epoch_seconds();

        let expiration_seconds = &auth_module.auth_config.activation_token_validity_minutes * 60;

        assert_eq!(username, token.data.username);

        match token.data.token_type {
            TokenType::AccountActivation => {}
            _ => panic!(),
        }
        assert!((before + expiration_seconds as i64) <= token.data.expire_at_epoch_seconds);
        assert!((after + expiration_seconds as i64) >= token.data.expire_at_epoch_seconds);

        assert!(auth_module.token_service.fetch_by_token_with_conn(conn, &token.data.token, true).await.is_ok());
        assert!(auth_module.token_service.delete_with_conn(conn, token.clone()).await.is_ok());
        assert!(auth_module.token_service.fetch_by_token_with_conn(conn, &token.data.token, true).await.is_err());
        Ok(())
    })
    .await
}

#[tokio_shared::test]
async fn should_validate_token_on_fetch() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;

    let c3p0 = auth_module.repo_manager.c3p0();

    c3p0.transaction(async |conn| {
        let token = NewRecord {
            data: TokenData {
                token: new_hyphenated_uuid(),
                expire_at_epoch_seconds: current_epoch_seconds() - 1,
                token_type: TokenType::ResetPassword,
                username: "test@test.com".to_owned(),
            },
        };

        let saved_token = conn.save(token).await?;

        assert!(auth_module.token_service.fetch_by_token_with_conn(conn, &saved_token.data.token, true).await.is_err());

        Ok(())
    })
    .await
}

#[tokio_shared::test]
async fn generate_and_save_token_should_lazily_delete_all_expired_tokens() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;
    let token_service = &auth_module.token_service;
    let c3p0 = auth_module.repo_manager.c3p0();

    c3p0.transaction::<_, LsAccountManagementError, _>(async |conn| {
        // Two stale tokens belonging to two different usernames — the sweep
        // must hit both, not just rows for the user we are minting next.
        let stale_a = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: 0,
                    token_type: TokenType::ResetPassword,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;
        let stale_b = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: 0,
                    token_type: TokenType::AccountActivation,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;

        // A non-expired token must survive the sweep.
        let live = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: current_epoch_seconds() + 3600,
                    token_type: TokenType::AccountActivation,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;

        // Minting a new token for an unrelated user triggers the sweep.
        let fresh = token_service
            .generate_and_save_token_with_conn(conn, new_hyphenated_uuid(), TokenType::AccountActivation)
            .await?;

        assert!(!conn.exists_by_id::<TokenData>(stale_a.id).await?);
        assert!(!conn.exists_by_id::<TokenData>(stale_b.id).await?);
        assert!(conn.exists_by_id::<TokenData>(live.id).await?);
        assert!(conn.exists_by_id::<TokenData>(fresh.id).await?);

        Ok(())
    })
    .await
}

#[tokio_shared::test]
async fn delete_expired_with_conn_should_remove_only_rows_below_threshold() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;
    let token_service = &auth_module.token_service;
    let c3p0 = auth_module.repo_manager.c3p0();

    c3p0.transaction::<_, LsAccountManagementError, _>(async |conn| {
        let now = current_epoch_seconds();

        let below = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: now - 100,
                    token_type: TokenType::AccountActivation,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;
        let at_threshold = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: now,
                    token_type: TokenType::AccountActivation,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;
        let above = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: now + 100,
                    token_type: TokenType::ResetPassword,
                    username: new_hyphenated_uuid(),
                },
            })
            .await?;

        let deleted = token_service.delete_expired_with_conn(conn, now).await?;
        assert!(deleted >= 1);

        // Strictly less-than threshold: row at exactly `now` survives.
        assert!(!conn.exists_by_id::<TokenData>(below.id).await?);
        assert!(conn.exists_by_id::<TokenData>(at_threshold.id).await?);
        assert!(conn.exists_by_id::<TokenData>(above.id).await?);

        Ok(())
    })
    .await
}

#[tokio_shared::test]
async fn should_return_all_tokens_by_username() -> Result<(), LsAccountManagementError> {
    let data = data(false).await;
    let auth_module = &data.0;
    let token_service = &auth_module.token_service;

    let c3p0 = auth_module.repo_manager.c3p0();

    let username_1 = new_hyphenated_uuid();
    let username_2 = new_hyphenated_uuid();

    c3p0.transaction(async |conn| {
        assert_eq!(0, token_service.fetch_all_by_username_with_conn(conn, &username_1).await?.len());
        assert_eq!(0, token_service.fetch_all_by_username_with_conn(conn, &username_2).await?.len());

        let token_1 = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: current_epoch_seconds() - 1,
                    token_type: TokenType::ResetPassword,
                    username: username_1.clone(),
                },
            })
            .await?;

        assert_eq!(1, token_service.fetch_all_by_username_with_conn(conn, &username_1).await?.len());

        let token_2 = conn
            .save(NewRecord {
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: current_epoch_seconds() - 1,
                    token_type: TokenType::AccountActivation,
                    username: username_1.clone(),
                },
            })
            .await?;

        assert_eq!(0, token_service.fetch_all_by_username_with_conn(conn, &username_2).await?.len());

        let user_1_tokens = token_service.fetch_all_by_username_with_conn(conn, &username_1).await?;
        assert_eq!(2, user_1_tokens.len());

        assert!(user_1_tokens.iter().any(|token| token.data.username == username_1 && token_1.id == token.id));
        assert!(user_1_tokens.iter().any(|token| token.data.username == username_1 && token_2.id == token.id));

        Ok(())
    })
    .await
}
