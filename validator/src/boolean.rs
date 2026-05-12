use crate::error::{ErrorDetails, ValidationError};

/// Validates whether the value is true
#[inline]
pub fn validate_is_true<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: bool) {
    if !val {
        error_details.add_detail(field_name.into(), ValidationError::MustBeTrue)
    }
}

/// Validates whether the value is false
#[inline]
pub fn validate_is_false<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: bool) {
    if val {
        error_details.add_detail(field_name.into(), ValidationError::MustBeFalse)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_is_true_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_is_true(&mut error_details, "name", true);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_is_true_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_is_true(&mut error_details, "name", false);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::MustBeTrue, error_details.details()["name"][0])
    }

    #[test]
    fn should_validate_is_false_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_is_false(&mut error_details, "name", false);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_is_false_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_is_false(&mut error_details, "name", true);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::MustBeFalse, error_details.details()["name"][0])
    }
}
