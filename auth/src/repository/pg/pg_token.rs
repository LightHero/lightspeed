use crate::model::token::{TokenData, TokenDataCodec, TokenModel};
use crate::repository::TokenRepository;
use c3p0::pg_async::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgTokenRepository {
    repo: PgC3p0JsonAsync<TokenData, TokenDataCodec>,
}

impl Deref for PgTokenRepository {
    type Target = PgC3p0JsonAsync<TokenData, TokenDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgTokenRepository {
    fn default() -> Self {
        PgTokenRepository {
            repo: C3p0JsonBuilder::new("LS_AUTH_TOKEN").build_with_codec(TokenDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl TokenRepository for PgTokenRepository {
    type Conn = PgConnectionAsync;

    async fn fetch_by_token(
        &self,
        conn: &mut PgConnectionAsync,
        token_string: &str,
    ) -> Result<TokenModel, LightSpeedError> {
        let sql = r#"
            select id, version, data from LS_AUTH_TOKEN
            where data ->> 'token' = $1
            limit 1
        "#;
        Ok(self.repo.fetch_one_with_sql(conn, sql, &[&token_string]).await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<TokenData>,
    ) -> Result<Model<TokenData>, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<TokenData>,
    ) -> Result<Model<TokenData>, LightSpeedError> {
        Ok(self.repo.delete(conn, model).await?)
    }
}
