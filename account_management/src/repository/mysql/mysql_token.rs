use crate::error::LsAccountManagementError;
use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use ::sqlx::AssertSqlSafe;
use c3p0::sqlx::*;
use c3p0::*;

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

    async fn fetch_by_token(
        &self,
        tx: &mut MySqlConnection,
        token_string: &str,
    ) -> Result<TokenModel, LsAccountManagementError> {
        Ok(TokenModel::query_with_tail(
            r#"
            where JSON_VALUE(data, '$.token' RETURNING CHAR(255)) = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(
        &self,
        tx: &mut MySqlConnection,
        username: &str,
    ) -> Result<Vec<TokenModel>, LsAccountManagementError> {
        Ok(TokenModel::query_with_tail(
            r#"
            where JSON_VALUE(data, '$.username' RETURNING CHAR(255)) = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(
        &self,
        tx: &mut MySqlConnection,
        model: NewRecord<TokenData>,
    ) -> Result<TokenModel, LsAccountManagementError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(
        &self,
        tx: &mut MySqlConnection,
        model: TokenModel,
    ) -> Result<TokenModel, LsAccountManagementError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_expired(
        &self,
        tx: &mut MySqlConnection,
        threshold_epoch_seconds: i64,
    ) -> Result<u64, LsAccountManagementError> {
        // Two-phase to avoid InnoDB deadlocks. A direct DELETE on the JSON
        // predicate — even backed by the LS_AM_TOKEN_EXPIRE_AT functional
        // index — takes next-key locks during the index range scan; rows
        // sharing an expire_at_epoch_seconds value have no tiebreaker so
        // concurrent sweeps lock them in non-deterministic order and
        // deadlock with error 1213. Selecting candidate ids (snapshot read,
        // no locks) and then DELETE WHERE id IN (...) bounds locking to
        // specific PK rows in a deterministic, sorted order. The functional
        // index keeps phase 1 fast.
        let select_sql = format!(
            "SELECT id FROM {} \
             WHERE JSON_VALUE(data, '$.expire_at_epoch_seconds' RETURNING SIGNED) < ? \
             ORDER BY id",
            <TokenData as DataType>::TABLE_NAME
        );
        let rows = query(AssertSqlSafe(select_sql)).bind(threshold_epoch_seconds).fetch_all(&mut *tx).await?;
        let ids: Vec<i64> = rows.iter().map(|row| row.try_get::<i64, _>(0)).collect::<Result<Vec<_>, _>>()?;

        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders = vec!["?"; ids.len()].join(",");
        let delete_sql = format!("DELETE FROM {} WHERE id IN ({})", <TokenData as DataType>::TABLE_NAME, placeholders);
        let mut q = query(AssertSqlSafe(delete_sql));
        for id in &ids {
            q = q.bind(id);
        }
        let res = q.execute(tx).await?;
        Ok(res.rows_affected())
    }
}
