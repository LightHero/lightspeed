use lightspeed_validator::must_match::validate_must_be_equals;
use lightspeed_validator::{ErrorDetails, Validable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl Validable for ResetPasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        Ok(())
    }
}
