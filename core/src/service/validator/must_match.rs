use crate::error::{ErrorDetail, ErrorDetails};

/// Validates that the 2 given fields match.
#[inline]
pub fn validate_must_match<A: Into<String>, B: Into<String>, T: Eq>(
    error_details: &mut ErrorDetails,
    a_name: A,
    a_value: T,
    b_name: B,
    b_value: T,
) {
    if !validator::validate_must_match(a_value, b_value) {
        let a_name = a_name.into();
        let b_name = b_name.into();
        error_details.add_detail(
            a_name.clone(),
            ErrorDetail::new("NOT_EQUALS", vec![a_name, b_name]),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_and_return_no_errors() {
        let mut error_details = ErrorDetails::new();
        validate_must_match(&mut error_details, "A", 1, "B", 1);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_and_return_errors() {
        let mut error_details = ErrorDetails::new();
        validate_must_match(&mut error_details, "A", 1, "B", 2);
        assert_eq!(1, error_details.details().len());
        assert_eq!(
            ErrorDetail::new("NOT_EQUALS", vec!["A".to_string(), "B".to_string()]),
            error_details.details()["A"][0]
        )
    }

}
