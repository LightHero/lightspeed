use crate::model::token::{TokenData, TokenModel};
use crate::repository::TokenRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgTokenRepository {
    repo: C3p0JsonPg<TokenData, DefaultJsonCodec>,
}

impl Deref for PgTokenRepository {
    type Target = C3p0JsonPg<TokenData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgTokenRepository {
    fn default() -> Self {
        PgTokenRepository {
            repo: C3p0JsonBuilder::new("AUTH_TOKEN")
                .build(),
        }
    }
}

impl TokenRepository for PgTokenRepository {
    type CONN = PgConnection;

    fn fetch_by_token(
        &self,
        conn: &PgConnection,
        token_string: &str,
    ) -> Result<TokenModel, LightSpeedError> {
        let sql = r#"
            select id, version, data from AUTH_TOKEN
            where AUTH_TOKEN.DATA ->> 'token' = $1
            limit 1
        "#;
        self.repo
            .fetch_one_with_sql(conn, sql, &[&token_string])?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No token found with code [{}]", token_string),
            })
    }

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<TokenData>,
    ) -> Result<Model<TokenData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<TokenData>) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}
