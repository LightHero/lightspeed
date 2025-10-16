use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct PgTokenRepository;

impl Default for PgTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgTokenRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TokenRepository for PgTokenRepository {
    type DB = Postgres;

    async fn fetch_by_token(&self, tx: &mut PgConnection, token_string: &str) -> Result<TokenModel, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data ->> 'token' = $1
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut PgConnection, username: &str) -> Result<Vec<TokenModel>, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data ->> 'username' = $1
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut PgConnection, model: NewRecord<TokenData>) -> Result<TokenModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut PgConnection, model: TokenModel) -> Result<TokenModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
