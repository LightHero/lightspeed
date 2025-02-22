use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::service::validator::must_match::validate_must_be_equals;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl Validable for ResetPasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LsError> {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        Ok(())
    }
}
