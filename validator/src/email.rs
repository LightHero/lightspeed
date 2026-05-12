use crate::error::{ErrorDetails, ValidationError};
use validator::ValidateEmail;

/// Validates whether the given string is an email based on Django `EmailValidator` and HTML5 specs
#[inline]
pub fn validate_email<S: Into<String>, T: ValidateEmail>(error_details: &mut ErrorDetails, field_name: S, val: T) {
    if !val.validate_email() {
        error_details.add_detail(field_name.into(), ValidationError::NotValidEmail)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_email(&mut error_details, "email", "ufoscout@gmail.com");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_email(&mut error_details, "email", "ufoscout_gmail.com");
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::NotValidEmail, error_details.details()["email"][0])
    }
}
