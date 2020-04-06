use crate::error::{ErrorDetail, ErrorDetails};

pub const NOT_VALID_URL: &str = "NOT_VALID_URL";

/// Validates whether the string given is a url
pub fn validate_url<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: &str) {
    if !validator::validate_url(val) {
        error_details.add_detail(field_name.into(), ErrorDetail::new(NOT_VALID_URL, vec![]))
    }
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::error::ErrorDetails;

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
        assert_eq!(
            error_details.details()["url"][0],
            ErrorDetail::new(NOT_VALID_URL, vec![]),
        )
    }
}
