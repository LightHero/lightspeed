use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct SqliteTokenRepository {}

impl Default for SqliteTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteTokenRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TokenRepository for SqliteTokenRepository {
    type DB = Sqlite;

    async fn fetch_by_token(&self, tx: &mut SqliteConnection, token_string: &str) -> Result<TokenModel, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data ->> '$.token' = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut SqliteConnection, username: &str) -> Result<Vec<TokenModel>, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data ->> '$.username' = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut SqliteConnection, model: NewRecord<TokenData>) -> Result<TokenModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut SqliteConnection, model: TokenModel) -> Result<TokenModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
