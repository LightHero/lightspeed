use crate::error::{ErrorDetail, ErrorDetails};
use std::fmt::Display;

pub const MUST_BE_LESS_OR_EQUAL: &str = "MUST_BE_LESS_OR_EQUAL";
pub const MUST_BE_LESS: &str = "MUST_BE_LESS";
pub const MUST_BE_GREATER_OR_EQUAL: &str = "MUST_BE_GREATER_OR_EQUAL";
pub const MUST_BE_GREATER: &str = "MUST_BE_GREATER";

/// Validates whether the value is less than or equal to max
#[inline]
pub fn validate_le<N: PartialOrd + Display, S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    max: N,
    val: N,
) {
    if val > max {
        error_details.add_detail(field_name.into(), ErrorDetail::new(MUST_BE_LESS_OR_EQUAL, vec![format!("{max}")]))
    }
}

/// Validates whether the value is less than max
#[inline]
pub fn validate_lt<N: PartialOrd + Display, S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    max: N,
    val: N,
) {
    if val >= max {
        error_details.add_detail(field_name.into(), ErrorDetail::new(MUST_BE_LESS, vec![format!("{max}")]))
    }
}

/// Validates whether the value is greater than or equal to min
#[inline]
pub fn validate_ge<N: PartialOrd + Display, S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    min: N,
    val: N,
) {
    if val < min {
        error_details.add_detail(field_name.into(), ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec![format!("{min}")]))
    }
}

/// Validates whether the value is greater than min
#[inline]
pub fn validate_gt<N: PartialOrd + Display, S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    min: N,
    val: N,
) {
    if val <= min {
        error_details.add_detail(field_name.into(), ErrorDetail::new(MUST_BE_GREATER, vec![format!("{min}")]))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::error::ErrorDetails;

    #[test]
    fn le_should_validate_number_less() {
        let mut error_details = ErrorDetails::default();
        validate_le(&mut error_details, "name", 10, 2);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn le_should_validate_number_equal() {
        let mut error_details = ErrorDetails::default();
        validate_le(&mut error_details, "name", 12, 12);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn le_should_not_validate_number_greater() {
        let mut error_details = ErrorDetails::default();
        validate_le(&mut error_details, "name", 12, 122);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(MUST_BE_LESS_OR_EQUAL, vec!["12".to_owned()]), error_details.details()["name"][0])
    }

    #[test]
    fn lt_should_validate_number_less() {
        let mut error_details = ErrorDetails::default();
        validate_lt(&mut error_details, "name", 1066, 12);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn lt_should_not_validate_number_equal() {
        let mut error_details = ErrorDetails::default();
        validate_lt(&mut error_details, "name", 12, 12);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(MUST_BE_LESS, vec!["12".to_owned()]), error_details.details()["name"][0])
    }

    #[test]
    fn lt_should_not_validate_number_greated() {
        let mut error_details = ErrorDetails::default();
        validate_lt(&mut error_details, "name", 14, 232);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(MUST_BE_LESS, vec!["14".to_owned()]), error_details.details()["name"][0])
    }

    #[test]
    fn ge_should_validate_number_greater() {
        let mut error_details = ErrorDetails::default();
        validate_ge(&mut error_details, "name", 10, 12);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn ge_should_validate_number_equal() {
        let mut error_details = ErrorDetails::default();
        validate_ge(&mut error_details, "name", 12, 12);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn ge_should_not_validate_number_less() {
        let mut error_details = ErrorDetails::default();
        validate_ge(&mut error_details, "name", 12, 2);
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec!["12".to_owned()]),
            error_details.details()["name"][0]
        )
    }

    #[test]
    fn gt_should_validate_number_greater() {
        let mut error_details = ErrorDetails::default();
        validate_gt(&mut error_details, "name", 10, 12);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn gt_should_not_validate_number_equal() {
        let mut error_details = ErrorDetails::default();
        validate_gt(&mut error_details, "name", 12, 12);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(MUST_BE_GREATER, vec!["12".to_owned()]), error_details.details()["name"][0])
    }

    #[test]
    fn gt_should_not_validate_number_less() {
        let mut error_details = ErrorDetails::default();
        validate_gt(&mut error_details, "name", 14, 2);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(MUST_BE_GREATER, vec!["14".to_owned()]), error_details.details()["name"][0])
    }
}
