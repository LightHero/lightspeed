use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::must_match::validate_must_match;
use lightspeed_core::service::validator::Validable;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl Validable for &ResetPasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_must_match(
            error_details,
            "password",
            &self.password,
            "password_confirm",
            &self.password_confirm,
        );
        Ok(())
    }
}
