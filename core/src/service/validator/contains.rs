use super::traits::Contains;
use crate::error::{ErrorDetails, ErrorDetail};

/// Validates whether the value contains the needle
/// The value needs to implement the Contains trait, which is implement on String, str and Hashmap<String>
/// by default.
#[inline]
pub fn validate_contains<S: Into<String>, T: Contains>(error_details: &mut ErrorDetails, field_name: S, val: T, needle: &str) {
    if !validate_contains_needle(val, needle) {
        error_details.add_detail(field_name, ErrorDetail::new("NOT_CONTAIN", vec![needle.to_string()]))
    }
}

#[inline]
fn validate_contains_needle<T: Contains>(val: T, needle: &str) -> bool {
    val.has_element(needle)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_validate_contains_string() {
        assert!(validate_contains_needle("hey", "e"));
    }

    #[test]
    fn test_validate_contains_string_can_fail() {
        assert_eq!(validate_contains_needle("hey", "o"), false);
    }

    #[test]
    fn test_validate_contains_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(validate_contains_needle(map, "hey"));
    }

    #[test]
    fn test_validate_contains_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert_eq!(validate_contains_needle(map, "bob"), false);
    }

    #[test]
    fn test_validate_contains_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(validate_contains_needle(test, "e"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(validate_contains_needle(test, "e"));
    }

    #[test]
    fn test_validate_contains_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert_eq!(validate_contains_needle(test, "o"), false);
        let test: Cow<'static, str> = String::from("hey").into();
        assert_eq!(validate_contains_needle(test, "o"), false);
    }

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_contains(&mut error_details, "name", "ufoscout", "ufo");
        assert!(error_details.details.is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_contains(&mut error_details, "name", "ufoscout", "alien");
        assert_eq!(1, error_details.details.len());
        assert_eq!(ErrorDetail::new("NOT_CONTAIN", vec!["alien".to_string()]), error_details.details["name"][0])
    }
}
