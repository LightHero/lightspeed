use crate::error::{ErrorDetail, ErrorDetails};
use validator::Contains;

/// Validates whether the value contains the needle
/// The value needs to implement the Contains trait, which is implement on String, str and Hashmap<String>
/// by default.
#[inline]
pub fn validate_contains<S: Into<String>, T: Contains>(
    error_details: &mut ErrorDetails,
    field_name: S,
    val: T,
    needle: &str,
) {
    if !validator::validate_contains(val, needle) {
        error_details.add_detail(
            field_name,
            ErrorDetail::new("NOT_CONTAIN", vec![needle.to_string()]),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::new();
        validate_contains(&mut error_details, "name", "ufoscout", "ufo");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::new();
        validate_contains(&mut error_details, "name", "ufoscout", "alien");
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new("NOT_CONTAIN", vec!["alien".to_string()]),
            error_details.details()["name"][0]
        )
    }
}
