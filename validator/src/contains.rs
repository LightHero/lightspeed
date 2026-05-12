use crate::error::{ErrorDetails, ValidationError};
use validator::ValidateContains;

/// Validates whether the value contains the needle.
/// The value needs to implement the ValidateContains trait, which is implemented
/// on String, str, and HashMap<String> by default.
#[inline]
pub fn validate_contains<S: Into<String>, T: ValidateContains>(
    error_details: &mut ErrorDetails,
    field_name: S,
    val: T,
    needle: &str,
) {
    if !val.validate_contains(needle) {
        error_details.add_detail(field_name.into(), ValidationError::MustContain { needle: needle.to_string() })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_contains(&mut error_details, "name", "ufoscout", "ufo");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_contains(&mut error_details, "name", "ufoscout", "alien");
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ValidationError::MustContain { needle: "alien".to_string() },
            error_details.details()["name"][0]
        )
    }
}
