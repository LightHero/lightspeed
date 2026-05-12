use crate::error::{ErrorDetails, ValidationError};
use validator::ValidateUrl;

/// Validates whether the string given is a URL
pub fn validate_url<S: Into<String>, T: ValidateUrl>(error_details: &mut ErrorDetails, field_name: S, val: T) {
    if !val.validate_url() {
        error_details.add_detail(field_name.into(), ValidationError::NotValidUrl)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_url(&mut error_details, "url", "http://www.gmail.com");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_url(&mut error_details, "url", "gmail");
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::NotValidUrl, error_details.details()["url"][0])
    }
}
