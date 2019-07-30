use c3p0::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub type TokenModel = Model<TokenData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub username: String,
    pub token_type: TokenType,
    pub expire_at_epoch: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccountActivation,
    ResetPassword,
}

#[derive(Clone)]
pub struct TokenRepository {
    repo: C3p0Json<TokenData, DefaultJsonCodec, PgJsonManager<TokenData, DefaultJsonCodec>>,
}

impl Deref for TokenRepository {
    type Target = C3p0Json<TokenData, DefaultJsonCodec, PgJsonManager<TokenData, DefaultJsonCodec>>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl TokenRepository {
    pub fn new() -> Self {
        TokenRepository {
            repo: C3p0JsonBuilder::new("AUTH_TOKEN")
                .with_data_field_name("data_json")
                .build(),
        }
    }

    pub fn fetch_by_token(
        &self,
        conn: &PgConnection,
        token_string: &str,
    ) -> Result<Option<TokenModel>, C3p0Error> {
        let sql = r#"
            select from AUTH_TOKEN
            where DATA_JSON ->> 'token' = ?
            limit 1
        "#;
        conn.fetch_one_option(sql, &[&token_string], |row| self.repo.json().to_model(row))
    }
}
