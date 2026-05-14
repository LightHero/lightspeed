pub mod auth_dto;
pub mod change_password_dto;
pub mod create_login_dto;
pub mod login_dto;
pub mod login_response_dto;
pub mod reset_password_dto;
pub mod send_new_activation_token_dto;
pub mod send_reset_password_dto;
pub mod token_dto;

use lightspeed_validator::error::{ErrorDetails, ValidationError};

pub const ERR_PASSWORD_TOO_SHORT: &str = "PASSWORD_TOO_SHORT";

pub(crate) fn validate_min_password_len<S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    password: &str,
    min: usize,
) {
    if password.len() < min {
        error_details.add_detail(
            field_name.into(),
            ValidationError::Custom { code: ERR_PASSWORD_TOO_SHORT.into(), params: vec![min.to_string()] },
        );
    }
}
