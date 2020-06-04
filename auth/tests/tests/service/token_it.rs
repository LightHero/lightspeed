use crate::{data, test};
use c3p0::*;
use lightspeed_auth::model::token::{TokenData, TokenType};
use lightspeed_auth::repository::AuthRepositoryManager;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};

#[test]
fn should_delete_token() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let c3p0 = auth_module.repo_manager.c3p0();
        let token_repo = auth_module.repo_manager.token_repo();

        let token = NewModel {
            version: 0,
            data: TokenData {
                token: new_hyphenated_uuid(),
                expire_at_epoch_seconds: 9999999999999,
                token_type: TokenType::RESET_PASSWORD,
                username: "test@test.com".to_owned(),
            },
        };

        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;

            let saved_token = token_repo.save(conn, token).await?;

            assert!(token_repo.exists_by_id(conn, &saved_token.id).await?);
            assert!(auth_module
                .token_service
                .delete_with_conn(conn, saved_token.clone())
                .await
                .is_ok());
            assert!(!token_repo.exists_by_id(conn, &saved_token.id).await?);

            Ok(())
        })
        .await
    })
}

#[test]
fn should_generate_token() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let c3p0 = auth_module.repo_manager.c3p0();
        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;
            let username = new_hyphenated_uuid();
            let token_type = TokenType::ACCOUNT_ACTIVATION;

            let before = current_epoch_seconds();
            let token = auth_module
                .token_service
                .generate_and_save_token_with_conn(conn, username.clone(), token_type.clone())
                .await?;
            let after = current_epoch_seconds();

            let expiration_seconds =
                &auth_module.auth_config.activation_token_validity_minutes * 60;

            assert_eq!(username, token.data.username);

            match token.data.token_type {
                TokenType::ACCOUNT_ACTIVATION => {}
                _ => assert!(false),
            }
            assert!((before + expiration_seconds) <= token.data.expire_at_epoch_seconds);
            assert!((after + expiration_seconds) >= token.data.expire_at_epoch_seconds);

            assert!(auth_module
                .token_service
                .fetch_by_token_with_conn(conn, &token.data.token, true)
                .await
                .is_ok());
            assert!(auth_module
                .token_service
                .delete_with_conn(conn, token.clone())
                .await
                .is_ok());
            assert!(auth_module
                .token_service
                .fetch_by_token_with_conn(conn, &token.data.token, true)
                .await
                .is_err());
            Ok(())
        })
        .await
    })
}

#[test]
fn should_validate_token_on_fetch() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;

        let c3p0 = auth_module.repo_manager.c3p0();
        let token_repo = auth_module.repo_manager.token_repo();

        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;
            let token = NewModel {
                version: 0,
                data: TokenData {
                    token: new_hyphenated_uuid(),
                    expire_at_epoch_seconds: current_epoch_seconds() - 1,
                    token_type: TokenType::RESET_PASSWORD,
                    username: "test@test.com".to_owned(),
                },
            };

            let saved_token = token_repo.save(conn, token).await?;

            assert!(auth_module
                .token_service
                .fetch_by_token_with_conn(conn, &saved_token.data.token, true)
                .await
                .is_err());

            Ok(())
        })
        .await
    })
}

#[test]
fn should_return_all_tokens_by_username() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let auth_module = &data.0;
        let token_service = &auth_module.token_service;

        let c3p0 = auth_module.repo_manager.c3p0();
        let token_repo = auth_module.repo_manager.token_repo();

        let username_1 = new_hyphenated_uuid();
        let username_2 = new_hyphenated_uuid();

        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;

            assert_eq!(
                0,
                token_service
                    .fetch_all_by_username_with_conn(conn, &username_1)
                    .await?
                    .len()
            );
            assert_eq!(
                0,
                token_service
                    .fetch_all_by_username_with_conn(conn, &username_2)
                    .await?
                    .len()
            );

            let token_1 = token_repo
                .save(
                    conn,
                    NewModel {
                        version: 0,
                        data: TokenData {
                            token: new_hyphenated_uuid(),
                            expire_at_epoch_seconds: current_epoch_seconds() - 1,
                            token_type: TokenType::RESET_PASSWORD,
                            username: username_1.clone(),
                        },
                    },
                )
                .await?;

            assert_eq!(
                1,
                token_service
                    .fetch_all_by_username_with_conn(conn, &username_1)
                    .await?
                    .len()
            );

            let token_2 = token_repo
                .save(
                    conn,
                    NewModel {
                        version: 0,
                        data: TokenData {
                            token: new_hyphenated_uuid(),
                            expire_at_epoch_seconds: current_epoch_seconds() - 1,
                            token_type: TokenType::ACCOUNT_ACTIVATION,
                            username: username_1.clone(),
                        },
                    },
                )
                .await?;

            assert_eq!(
                0,
                token_service
                    .fetch_all_by_username_with_conn(conn, &username_2)
                    .await?
                    .len()
            );

            let user_1_tokens = token_service
                .fetch_all_by_username_with_conn(conn, &username_1)
                .await?;
            assert_eq!(2, user_1_tokens.len());

            assert!(user_1_tokens
                .iter()
                .any(|token| token.data.username == username_1 && token_1.id == token.id));
            assert!(user_1_tokens
                .iter()
                .any(|token| token.data.username == username_1 && token_2.id == token.id));

            Ok(())
        })
        .await
    })
}
