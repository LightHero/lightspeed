use crate::model::token::{TokenData, TokenDataCodec, TokenModel};
use crate::repository::TokenRepository;
use ::sqlx::{MySql, Transaction};
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct MySqlTokenRepository {
    repo: SqlxMySqlC3p0Json<u64, TokenData, TokenDataCodec>,
}

impl Deref for MySqlTokenRepository {
    type Target = SqlxMySqlC3p0Json<u64, TokenData, TokenDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for MySqlTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlTokenRepository {
    pub fn new() -> Self {
        Self { repo: SqlxMySqlC3p0JsonBuilder::new("LS_AUTH_TOKEN").build_with_codec(TokenDataCodec {}) }
    }
}

impl TokenRepository for MySqlTokenRepository {
    type Tx<'a> = Transaction<'a, MySql>;

    async fn fetch_by_token(&self, tx: &mut Self::Tx<'_>, token_string: &str) -> Result<TokenModel, LsError> {
        let sql = &format!(
            r#"
            {}
            where data -> '$.token' = ?
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_with_sql(tx, ::sqlx::query(sql).bind(token_string)).await?)
    }

    async fn fetch_by_username(&self, tx: &mut Self::Tx<'_>, username: &str) -> Result<Vec<TokenModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where data -> '$.username' = ?
        "#,
            self.queries().find_base_sql_query
        );

        Ok(self.repo.fetch_all_with_sql(tx, ::sqlx::query(&sql).bind(username)).await?)
    }

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<TokenData>) -> Result<TokenModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: TokenModel) -> Result<TokenModel, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
