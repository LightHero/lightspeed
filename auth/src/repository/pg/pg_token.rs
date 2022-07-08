use crate::model::token::{TokenData, TokenDataCodec, TokenModel};
use crate::repository::TokenRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgTokenRepository {
    repo: PgC3p0Json<TokenData, TokenDataCodec>,
}

impl Deref for PgTokenRepository {
    type Target = PgC3p0Json<TokenData, TokenDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgTokenRepository {
    fn default() -> Self {
        PgTokenRepository { repo: C3p0JsonBuilder::new("LS_AUTH_TOKEN").build_with_codec(TokenDataCodec {}) }
    }
}

#[async_trait::async_trait]
impl TokenRepository for PgTokenRepository {
    type Conn = PgConnection;

    async fn fetch_by_token(&self, conn: &mut PgConnection, token_string: &str) -> Result<TokenModel, LightSpeedError> {
        let sql = format!(r#"
            {}
            where data ->> 'token' = $1
            limit 1
        "#, self.queries().find_base_sql_query);
        Ok(self.repo.fetch_one_with_sql(conn, &sql, &[&token_string]).await?)
    }

    async fn fetch_by_username(
        &self,
        conn: &mut PgConnection,
        username: &str,
    ) -> Result<Vec<TokenModel>, LightSpeedError> {
        let sql = format!(r#"
            {}
            where data ->> 'username' = $1
        "#, self.queries().find_base_sql_query);
        Ok(self.repo.fetch_all_with_sql(conn, &sql, &[&username]).await?)
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
