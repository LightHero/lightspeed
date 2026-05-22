use c3p0::*;
use serde::{Deserialize, Serialize};

pub type TokenModel = Record<TokenData>;

pub const ERR_TOKEN_EXPIRED: &str = "expired";

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub username: String,
    pub token_type: TokenType,
    pub expire_at_epoch_seconds: i64,
}

impl DataType for TokenData {
    const TABLE_NAME: &'static str = "LS_AM_TOKEN";
    type CODEC = TokenDataCodec;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccountActivation,
    ResetPassword,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum TokenDataCodec {
    V1(TokenData),
}

impl Codec<TokenData> for TokenDataCodec {
    fn encode(data: TokenData) -> Self {
        TokenDataCodec::V1(data)
    }

    fn decode(data: Self) -> TokenData {
        match data {
            TokenDataCodec::V1(data) => data,
        }
    }
}
