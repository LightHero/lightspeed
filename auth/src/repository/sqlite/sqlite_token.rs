use crate::model::token::{TokenData, TokenDataCodec, TokenModel};
use crate::repository::TokenRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct SqliteTokenRepository {
    repo: SqlxSqliteC3p0Json<u64, TokenData, TokenDataCodec>,
}

impl Deref for SqliteTokenRepository {
    type Target = SqlxSqliteC3p0Json<u64, TokenData, TokenDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for SqliteTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteTokenRepository {
    pub fn new() -> Self {
        Self { repo: SqlxSqliteC3p0JsonBuilder::new("LS_AUTH_TOKEN").build_with_codec(TokenDataCodec {}) }
    }
}

impl TokenRepository for SqliteTokenRepository {
    type Tx = SqliteTx;

    async fn fetch_by_token(&self, tx: &mut Self::Tx, token_string: &str) -> Result<TokenModel, LsError> {
        let sql = &format!(
            r#"
            {}
            where data ->> '$.token' = ?
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_with_sql(tx, ::sqlx::query(sql).bind(token_string)).await?)
    }

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<Vec<TokenModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where data ->> '$.username' = ?
        "#,
            self.queries().find_base_sql_query
        );

        Ok(self.repo.fetch_all_with_sql(tx, ::sqlx::query(&sql).bind(username)).await?)
    }

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<TokenData>) -> Result<TokenModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx, model: TokenModel) -> Result<TokenModel, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
