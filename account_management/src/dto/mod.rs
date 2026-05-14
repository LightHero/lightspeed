pub mod auth_dto;
pub mod change_password_dto;
pub mod create_login_dto;
pub mod login_dto;
pub mod login_response_dto;
pub mod reset_password_dto;
pub mod send_new_activation_token_dto;
pub mod send_reset_password_dto;
pub mod token_dto;

use lightspeed_core::error::ErrorDetails;
use lightspeed_validator::FieldValidator;
use lightspeed_validator::email::{EmailError, EmailValidator};

pub const ERR_PASSWORD_TOO_SHORT: &str = "PASSWORD_TOO_SHORT";
pub const ERR_MUST_MATCH: &str = "MUST_MATCH";
pub const ERR_MUST_BE_TRUE: &str = "MUST_BE_TRUE";
pub const ERR_EMAIL: &str = "EMAIL_NOT_VALID";
pub const ERR_NOT_UNIQUE: &str = "NOT_UNIQUE";

pub(crate) fn validate_min_password_len<S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    password: &str,
    min: usize,
) {
    if password.len() < min {
        error_details.add_detail(field_name.into(), (ERR_PASSWORD_TOO_SHORT, vec![min.to_string()]));
    }
}

pub(crate) fn validate_must_be_equals<S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_a_name: S,
    field_a: &str,
    field_b_name: &'static str,
    field_b: &str,
) {
    if field_a != field_b {
        error_details.add_detail(field_a_name.into(), (ERR_MUST_MATCH, vec![field_b_name.to_string()]));
    }
}

pub(crate) fn validate_is_true<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, value: bool) {
    if !value {
        error_details.add_detail(field_name.into(), ERR_MUST_BE_TRUE);
    }
}

pub(crate) fn validate_email<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, value: &str) {
    let result: Result<(), EmailError> = EmailValidator.validate(&value, &());
    if result.is_err() {
        error_details.add_detail(field_name.into(), ERR_EMAIL);
    }
}
