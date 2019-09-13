use crate::error::{ErrorDetail, ErrorDetails};

pub const MUST_BE_LESS_OR_EQUAL: &str = "MUST_BE_LESS_OR_EQUAL";
pub const MUST_BE_LESS: &str = "MUST_BE_LESS";
pub const MUST_BE_GREATER_OR_EQUAL: &str = "MUST_BE_GREATER_OR_EQUAL";
pub const MUST_BE_GREATER: &str = "MUST_BE_GREATER";

/// Validates whether the size is less than or equal to max
#[inline]
pub fn validate_number_le<S: Into<String>>(
    error_details: &ErrorDetails,
    field_name: S,
    max: usize,
    val: usize,
) {
    if val > max {
        error_details.add_detail(
            field_name,
            ErrorDetail::new(MUST_BE_LESS_OR_EQUAL, vec![format!("{}", max)]),
        )
    }
}

/// Validates whether the size is less than max
#[inline]
pub fn validate_number_lt<S: Into<String>>(
    error_details: &ErrorDetails,
    field_name: S,
    max: usize,
    val: usize,
) {
    if val >= max {
        error_details.add_detail(
            field_name,
            ErrorDetail::new(MUST_BE_LESS, vec![format!("{}", max)]),
        )
    }
}

/// Validates whether the size is greater than or equal to min
#[inline]
pub fn validate_number_ge<S: Into<String>>(
    error_details: &ErrorDetails,
    field_name: S,
    min: usize,
    val: usize,
) {
    if val < min {
        error_details.add_detail(
            field_name,
            ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec![format!("{}", min)]),
        )
    }
}

/// Validates whether the size is greater than min
#[inline]
pub fn validate_number_gt<S: Into<String>>(
    error_details: &ErrorDetails,
    field_name: S,
    min: usize,
    val: usize,
) {
    if val <= min {
        error_details.add_detail(
            field_name,
            ErrorDetail::new(MUST_BE_GREATER, vec![format!("{}", min)]),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn le_should_validate_number_less() {
        let error_details = ErrorDetails::new();
        validate_number_le(&error_details, "name", 10, 2);
        assert!(error_details.details().clone().borrow().is_empty())
    }

    #[test]
    fn le_should_validate_number_equal() {
        let error_details = ErrorDetails::new();
        validate_number_le(&error_details, "name", 12, 12);
        assert!(error_details.details().clone().borrow().is_empty())
    }

    #[test]
    fn le_should_not_validate_number_greater() {
        let error_details = ErrorDetails::new();
        validate_number_le(&error_details, "name", 12, 122);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_LESS_OR_EQUAL, vec!["12".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

    #[test]
    fn lt_should_validate_number_less() {
        let error_details = ErrorDetails::new();
        validate_number_lt(&error_details, "name", 1066, 12);
        assert!(error_details.details().borrow().is_empty())
    }

    #[test]
    fn lt_should_not_validate_number_equal() {
        let error_details = ErrorDetails::new();
        validate_number_lt(&error_details, "name", 12, 12);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_LESS, vec!["12".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

    #[test]
    fn lt_should_not_validate_number_greated() {
        let error_details = ErrorDetails::new();
        validate_number_lt(&error_details, "name", 14, 232);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_LESS, vec!["14".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

    #[test]
    fn ge_should_validate_number_greater() {
        let error_details = ErrorDetails::new();
        validate_number_ge(&error_details, "name", 10, 12);
        assert!(error_details.details().borrow().is_empty())
    }

    #[test]
    fn ge_should_validate_number_equal() {
        let error_details = ErrorDetails::new();
        validate_number_ge(&error_details, "name", 12, 12);
        assert!(error_details.details().borrow().is_empty())
    }

    #[test]
    fn ge_should_not_validate_number_less() {
        let error_details = ErrorDetails::new();
        validate_number_ge(&error_details, "name", 12, 2);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec!["12".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

    #[test]
    fn gt_should_validate_number_greater() {
        let error_details = ErrorDetails::new();
        validate_number_gt(&error_details, "name", 10, 12);
        assert!(error_details.details().borrow().is_empty())
    }

    #[test]
    fn gt_should_not_validate_number_equal() {
        let error_details = ErrorDetails::new();
        validate_number_gt(&error_details, "name", 12, 12);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_GREATER, vec!["12".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

    #[test]
    fn gt_should_not_validate_number_less() {
        let error_details = ErrorDetails::new();
        validate_number_gt(&error_details, "name", 14, 2);
        assert_eq!(1, error_details.details().borrow().len());
        assert_eq!(
            ErrorDetail::new(MUST_BE_GREATER, vec!["14".to_owned()]),
            error_details.details().clone().borrow()["name"][0]
        )
    }

}
