use crate::error::{ErrorDetail, ErrorDetails};

/// Validates whether the given string is an email based on Django `EmailValidator` and HTML5 specs
#[inline]
pub fn validate_email<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: &str) {
    if !validator::validate_email(val) {
        error_details.add_detail(field_name, ErrorDetail::new("NOT_VALID", vec![]))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::new();
        validate_email(&mut error_details, "email", "ufoscout@gmail.com");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::new();
        validate_email(&mut error_details, "email", "ufoscout_gmail.com");
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new("NOT_VALID", vec![]),
            error_details.details()["email"][0]
        )
    }

}
