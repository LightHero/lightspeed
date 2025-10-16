use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct MySqlTokenRepository {}

impl Default for MySqlTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlTokenRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TokenRepository for MySqlTokenRepository {
    type DB = MySql;

    async fn fetch_by_token(&self, tx: &mut MySqlConnection, token_string: &str) -> Result<TokenModel, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data -> '$.token' = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut MySqlConnection, username: &str) -> Result<Vec<TokenModel>, LsError> {
        Ok(TokenModel::query_with(
            r#"
            where data -> '$.username' = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut MySqlConnection, model: NewRecord<TokenData>) -> Result<TokenModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut MySqlConnection, model: TokenModel) -> Result<TokenModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
