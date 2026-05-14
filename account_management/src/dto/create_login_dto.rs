use crate::dto::{validate_email, validate_is_true, validate_must_be_equals};
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateLoginDto {
    pub username: Option<String>,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub language: Language,
    pub data: HashMap<String, String>,
    pub accept_privacy_policy: bool,
}

impl CreateLoginDto {
    pub(crate) fn validate(&self, error_details: &mut ErrorDetails) {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        validate_is_true(error_details, "accept_privacy_policy", self.accept_privacy_policy);
        validate_email(error_details, "email", &self.email);
    }
}
