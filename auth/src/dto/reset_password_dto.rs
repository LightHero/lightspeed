use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::service::validator::must_match::validate_must_be_equals;
use serde::{Deserialize, Serialize};

use crate::dto::{MIN_PASSWORD_LEN, validate_min_password_len};

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl Validable for ResetPasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LsError> {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        validate_min_password_len(error_details, "password", &self.password, MIN_PASSWORD_LEN);
        Ok(())
    }
}
