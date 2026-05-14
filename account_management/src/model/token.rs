use c3p0::*;
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::utils::current_epoch_seconds;
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
    const TABLE_NAME: &'static str = "LS_AUTH_TOKEN";
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

impl TokenData {
    pub(crate) fn validate(&self, error_details: &mut ErrorDetails) {
        if current_epoch_seconds() > self.expire_at_epoch_seconds {
            error_details.add_detail("expire_at_epoch", ERR_TOKEN_EXPIRED);
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use lightspeed_core::error::RootErrorDetails;

    #[test]
    pub fn token_not_expired_should_be_valid() {
        let token = TokenData {
            token: "".to_owned(),
            token_type: TokenType::AccountActivation,
            username: "".to_owned(),
            expire_at_epoch_seconds: current_epoch_seconds() + 1000,
        };

        let mut details = ErrorDetails::Root(RootErrorDetails::default());
        token.validate(&mut details);
        assert!(details.details().is_empty());
    }

    #[test]
    pub fn token_expired_should_not_be_valid() {
        let token = TokenData {
            token: "".to_owned(),
            token_type: TokenType::AccountActivation,
            username: "".to_owned(),
            expire_at_epoch_seconds: current_epoch_seconds() - 1000,
        };

        let mut details = ErrorDetails::Root(RootErrorDetails::default());
        token.validate(&mut details);
        assert_eq!(ERR_TOKEN_EXPIRED, details.details()["expire_at_epoch"][0]);
    }
}
