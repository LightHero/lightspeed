use c3p0::Model;
use serde::{Deserialize, Serialize};

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
