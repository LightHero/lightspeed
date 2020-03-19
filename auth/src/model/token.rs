use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::utils::current_epoch_seconds;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

pub type TokenModel = Model<TokenData>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    pub token: String,
    pub username: String,
    pub token_type: TokenType,
    pub expire_at_epoch_seconds: i64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    ACCOUNT_ACTIVATION,
    RESET_PASSWORD,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum TokenDataVersioning<'a> {
    V1(Cow<'a, TokenData>),
}

#[derive(Clone)]
pub struct TokenDataCodec {}

impl JsonCodec<TokenData> for TokenDataCodec {
    fn from_value(&self, value: Value) -> Result<TokenData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            TokenDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn to_value(&self, data: &TokenData) -> Result<Value, C3p0Error> {
        serde_json::to_value(TokenDataVersioning::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

impl Validable for &TokenData {
    fn validate<E: ErrorDetails>(&self, error_details: &mut E) -> Result<(), LightSpeedError> {
        if current_epoch_seconds() > self.expire_at_epoch_seconds {
            error_details.add_detail("expire_at_epoch".into(), "expired".into());
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use lightspeed_core::service::validator::Validator;

    #[test]
    pub fn token_not_expired_should_be_valid() {
        let token = TokenData {
            token: "".to_owned(),
            token_type: TokenType::ACCOUNT_ACTIVATION,
            username: "".to_owned(),
            expire_at_epoch_seconds: current_epoch_seconds() + 1000,
        };

        assert!(Validator::validate(&token).is_ok())
    }

    #[test]
    pub fn token_expired_should_not_be_valid() {
        let token = TokenData {
            token: "".to_owned(),
            token_type: TokenType::ACCOUNT_ACTIVATION,
            username: "".to_owned(),
            expire_at_epoch_seconds: current_epoch_seconds() - 1000,
        };

        let result = Validator::validate(&token);

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("expired", details.details["expire_at_epoch"][0])
            }
            _ => assert!(false),
        }
    }
}
