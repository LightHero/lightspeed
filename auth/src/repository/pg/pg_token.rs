use crate::model::token::{TokenData, TokenDataCodec, TokenModel};
use crate::repository::TokenRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgTokenRepository<Id: IdType> {
    repo: SqlxPgC3p0Json<Id, TokenData, TokenDataCodec>,
}

impl <Id: IdType> Deref for PgTokenRepository<Id> {
    type Target = SqlxPgC3p0Json<Id, TokenData, TokenDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl <Id: IdType> Default for PgTokenRepository<Id> {
    fn default() -> Self {
        PgTokenRepository { repo: C3p0JsonBuilder::new("LS_AUTH_TOKEN").build_with_codec(TokenDataCodec {}) }
    }
}

#[async_trait::async_trait]
impl <Id: IdType> TokenRepository<Id> for PgTokenRepository<Id> {
    type Tx = PgTx;

    async fn fetch_by_token(&self, tx: &mut Self::Tx, token_string: &str) -> Result<TokenModel<Id>, LsError> {
        let sql = &format!(
            r#"
            {}
            where data ->> 'token' = $1
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_with_sql(tx, ::sqlx::query(&sql).bind(token_string)).await?)
    }

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<Vec<TokenModel<Id>>, LsError> {
        let sql = format!(
            r#"
            {}
            where data ->> 'username' = $1
        "#,
            self.queries().find_base_sql_query
        );

        Ok(self.repo.fetch_all_with_sql(tx, ::sqlx::query(&sql).bind(username)).await?)
    }

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<TokenData>) -> Result<Model<Id, TokenData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx, model: Model<Id, TokenData>) -> Result<Model<Id, TokenData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
