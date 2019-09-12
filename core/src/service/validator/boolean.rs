use crate::error::{ErrorDetail, ErrorDetails};

/// Validates whether the value is true
#[inline]
pub fn validate_is_true<S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    val: bool,
) {
    if !val {
        error_details.add_detail(field_name, ErrorDetail::new("MUST_BE_TRUE", vec![]))
    }
}

/// Validates whether the value is false
#[inline]
pub fn validate_is_false<S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    val: bool,
) {
    if val {
        error_details.add_detail(field_name, ErrorDetail::new("MUST_BE_FALSE", vec![]))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_is_true_and_return_no_errors() {
        let mut error_details = ErrorDetails::new();
        validate_is_true(&mut error_details, "name", true);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_is_true_and_return_errors() {
        let mut error_details = ErrorDetails::new();
        validate_is_true(&mut error_details, "name", false);
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new("MUST_BE_TRUE", vec![]),
            error_details.details()["name"][0]
        )
    }

    #[test]
    fn should_validate_is_false_and_return_no_errors() {
        let mut error_details = ErrorDetails::new();
        validate_is_false(&mut error_details, "name", false);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_is_false_and_return_errors() {
        let mut error_details = ErrorDetails::new();
        validate_is_false(&mut error_details, "name", true);
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new("MUST_BE_FALSE", vec![]),
            error_details.details()["name"][0]
        )
    }
}
