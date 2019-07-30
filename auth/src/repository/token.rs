use c3p0::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub username: String,
    pub token_type: TokenType,
    pub expire_at_epoch: i64
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccountActivation,
    ResetPassword
}

pub struct TokenRepository {
    repo: C3p0Json<TokenData, DefaultJsonCodec, PgJsonManager<TokenData, DefaultJsonCodec>>
}

impl TokenRepository {
    pub fn new() -> Self {
        TokenRepository {
            repo: C3p0JsonBuilder::new("AUTH_TOKEN").build()
        }
    }
}