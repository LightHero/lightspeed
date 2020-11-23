use crate::error::{ErrorDetail, ErrorDetails};

pub const NOT_VALID_IP: &str = "NOT_VALID_IP";

/// Validates whether the given string is an IP V4
#[inline]
pub fn validate_ip_v4<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: &str) {
    if !validator::validate_ip_v4(val) {
        error_details.add_detail(field_name.into(), ErrorDetail::new(NOT_VALID_IP, vec![]))
    }
}

/// Validates whether the given string is an IP V6
#[inline]
pub fn validate_ip_v6<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: &str) {
    if !validator::validate_ip_v6(val) {
        error_details.add_detail(field_name.into(), ErrorDetail::new(NOT_VALID_IP, vec![]))
    }
}

/// Validates whether the given string is an IP
#[inline]
pub fn validate_ip<S: Into<String>>(error_details: &mut ErrorDetails, field_name: S, val: &str) {
    if !validator::validate_ip(val) {
        error_details.add_detail(field_name.into(), ErrorDetail::new(NOT_VALID_IP, vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetails;

    #[test]
    fn should_validate_generic_ip_v6_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip(&mut error_details, "ip", "2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_generic_ip_v4_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip(&mut error_details, "ip", "127.0.0.1");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_ip_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip(&mut error_details, "ip", "127.0.0.1.1");
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(NOT_VALID_IP, vec![]), error_details.details()["ip"][0])
    }

    #[test]
    fn should_validate_ip4_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip_v4(&mut error_details, "ip", "127.0.0.1");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_ip4_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip_v4(&mut error_details, "ip", "127.0.0.1.1");
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(NOT_VALID_IP, vec![]), error_details.details()["ip"][0])
    }

    #[test]
    fn should_validate_ip6_and_return_no_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip_v6(&mut error_details, "ip", "2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_ip6_and_return_errors() {
        let mut error_details = ErrorDetails::default();
        validate_ip_v6(&mut error_details, "ip", "2001:0db8:85a3:0000:0000:8a2e:0370:7334:abc");
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(NOT_VALID_IP, vec![]), error_details.details()["ip"][0])
    }
}
