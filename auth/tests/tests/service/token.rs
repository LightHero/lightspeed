use crate::test;
use c3p0::NewModel;
use lightspeed_auth::repository::token::{TokenData, TokenModel, TokenType};

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
        assert_eq!(1, auth_module.token_service.delete(saved_token.clone())?);
        assert!(!auth_module
            .token_repo
            .exists_by_id(&c3p0.connection()?, &saved_token.id)?);

        Ok(())
    });
}
