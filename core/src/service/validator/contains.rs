use crate::error::{ErrorDetail, ErrorDetails};
use validator::Contains;

pub const MUST_CONTAIN: &str = "MUST_CONTAIN";

/// Validates whether the value contains the needle
/// The value needs to implement the Contains trait, which is implement on String, str and Hashmap<String>
/// by default.
#[inline]
pub fn validate_contains<S: Into<String>, T: Contains>(
    error_details: &ErrorDetails,
    field_name: S,
    val: T,
    needle: &str,
) {
    if !validator::validate_contains(val, needle) {
        error_details.add_detail(
            field_name,
            ErrorDetail::new(MUST_CONTAIN, vec![needle.to_string()]),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let error_details = ErrorDetails::new();
        validate_contains(&error_details, "name", "ufoscout", "ufo");
        assert!(error_details.details().borrow().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let error_details = ErrorDetails::new();
        validate_contains(&error_details, "name", "ufoscout", "alien");
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_CONTAIN, vec!["alien".to_string()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }
}
