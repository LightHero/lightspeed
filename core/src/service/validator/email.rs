use validator::ValidateEmail;

use crate::error::{ErrorDetail, ErrorDetails};

pub const NOT_VALID_EMAIL: &str = "NOT_VALID_EMAIL";

/// Validates whether the given string is an email based on Django `EmailValidator` and HTML5 specs
#[inline]
pub fn validate_email<S: Into<String>, T: ValidateEmail>(error_details: &mut ErrorDetails, field_name: S, val: T) {
    if !val.validate_email() {
        error_details.add_detail(field_name.into(), ErrorDetail::new(NOT_VALID_EMAIL, vec![]))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::error::ErrorDetails;

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
        assert_eq!(ErrorDetail::new(NOT_VALID_EMAIL, vec![]), error_details.details()["email"][0])
    }
}
