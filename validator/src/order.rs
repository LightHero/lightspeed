use crate::error::{ErrorDetails, ValidationError};
use std::fmt::Display;

/// Validates whether the value is less than or equal to max
#[inline]
pub fn validate_le<N: PartialOrd + Display, S: Into<String>>(
    error_details: &mut ErrorDetails,
    field_name: S,
    max: N,
    val: N,
) {
    if val > max {
        error_details.add_detail(field_name.into(), ValidationError::MustBeLessOrEqual { max: format!("{max}") })
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
        error_details.add_detail(field_name.into(), ValidationError::MustBeLess { max: format!("{max}") })
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
        error_details.add_detail(field_name.into(), ValidationError::MustBeGreaterOrEqual { min: format!("{min}") })
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
        error_details.add_detail(field_name.into(), ValidationError::MustBeGreater { min: format!("{min}") })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
        assert_eq!(
            ValidationError::MustBeLessOrEqual { max: "12".to_owned() },
            error_details.details()["name"][0]
        )
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
        assert_eq!(ValidationError::MustBeLess { max: "12".to_owned() }, error_details.details()["name"][0])
    }

    #[test]
    fn lt_should_not_validate_number_greated() {
        let mut error_details = ErrorDetails::default();
        validate_lt(&mut error_details, "name", 14, 232);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::MustBeLess { max: "14".to_owned() }, error_details.details()["name"][0])
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
            ValidationError::MustBeGreaterOrEqual { min: "12".to_owned() },
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
        assert_eq!(ValidationError::MustBeGreater { min: "12".to_owned() }, error_details.details()["name"][0])
    }

    #[test]
    fn gt_should_not_validate_number_less() {
        let mut error_details = ErrorDetails::default();
        validate_gt(&mut error_details, "name", 14, 2);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::MustBeGreater { min: "14".to_owned() }, error_details.details()["name"][0])
    }
}
