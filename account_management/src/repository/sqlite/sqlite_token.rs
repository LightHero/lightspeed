use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use ::sqlx::AssertSqlSafe;
use c3p0::sqlx::*;
use c3p0::*;
use crate::error::LsAccountManagerError;

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

    async fn fetch_by_token(&self, tx: &mut SqliteConnection, token_string: &str) -> Result<TokenModel, LsAccountManagerError> {
        Ok(TokenModel::query_with_tail(
            r#"
            where data ->> '$.token' = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut SqliteConnection, username: &str) -> Result<Vec<TokenModel>, LsAccountManagerError> {
        Ok(TokenModel::query_with_tail(
            r#"
            where data ->> '$.username' = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut SqliteConnection, model: NewRecord<TokenData>) -> Result<TokenModel, LsAccountManagerError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut SqliteConnection, model: TokenModel) -> Result<TokenModel, LsAccountManagerError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_expired(&self, tx: &mut SqliteConnection, threshold_epoch_seconds: i64) -> Result<u64, LsAccountManagerError> {
        let sql = format!(
            "DELETE FROM {} WHERE CAST(data ->> '$.expire_at_epoch_seconds' AS INTEGER) < ?",
            <TokenData as DataType>::TABLE_NAME
        );
        let res = query(AssertSqlSafe(sql)).bind(threshold_epoch_seconds).execute(tx).await?;
        Ok(res.rows_affected())
    }
}
