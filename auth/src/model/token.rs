use c3p0::Model;
use serde::{Deserialize, Serialize};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::error::{LightSpeedError, ErrorDetails};
use lightspeed_core::utils::current_epoch_seconds;

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

impl Validable for &TokenData {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        if current_epoch_seconds() > self.expire_at_epoch {
            error_details.add_detail("expire_at_epoch", "expired");
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
            token_type: TokenType::AccountActivation,
            username: "".to_owned(),
            expire_at_epoch: current_epoch_seconds() + 1000
        };

        assert!(Validator::validate(&token).is_ok())
    }

    #[test]
    pub fn token_expired_should_not_be_valid() {
        let token = TokenData {
            token: "".to_owned(),
            token_type: TokenType::AccountActivation,
            username: "".to_owned(),
            expire_at_epoch: current_epoch_seconds() - 1000
        };

        let result = Validator::validate(&token);

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError {details}) => {
                assert_eq!("expired", details.details["expire_at_epoch"][0])
            },
            _ => assert!(false)
        }
    }
}