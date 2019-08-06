use c3p0::*;
use std::ops::Deref;
use crate::model::token::{TokenData, TokenModel};


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
            select id, version, data_json from AUTH_TOKEN
            where AUTH_TOKEN.DATA_JSON ->> 'token' = $1
            limit 1
        "#;
        conn.fetch_one_option(sql, &[&token_string], |row| self.repo.json().to_model(row))
    }
}
