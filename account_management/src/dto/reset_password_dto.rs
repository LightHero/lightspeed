use crate::dto::validate_must_be_equals;
use lightspeed_core::error::ErrorDetails;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl ResetPasswordDto {
    pub(crate) fn validate(&self, error_details: &mut ErrorDetails) {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
    }
}
