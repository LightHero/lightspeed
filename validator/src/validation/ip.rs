use std::fmt::Display;
use std::net::IpAddr;
use std::str::FromStr;

use crate::FieldValidator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpKind {
    Any,
    V4,
    V6,
}

impl Display for IpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpKind::Any => write!(f, "any"),
            IpKind::V4 => write!(f, "v4"),
            IpKind::V6 => write!(f, "v6"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpError {
    pub kind: IpKind,
}

impl Display for IpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ip (expected: {})", self.kind)
    }
}

/// Validates that a string-compatible value parses as an IP address of the
/// configured `kind`.
pub struct IpValidator {
    pub kind: IpKind,
}

impl<S: AsRef<str>, E: From<IpError>, Ctx> FieldValidator<S, E, Ctx> for IpValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let parsed = IpAddr::from_str(value.as_ref());
        let ok = match self.kind {
            IpKind::Any => parsed.is_ok(),
            IpKind::V4 => parsed.is_ok_and(|ip| ip.is_ipv4()),
            IpKind::V6 => parsed.is_ok_and(|ip| ip.is_ipv6()),
        };
        if ok { Ok(()) } else { Err(IpError { kind: self.kind }.into()) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    #[test]
    fn ip_any_accepts_valid_v4_and_v6() {
        let validator = IpValidator { kind: IpKind::Any };
        assert_eq!(validator.validate(&"1.1.1.1", &()), OK);
        assert_eq!(validator.validate(&"255.0.0.0", &()), OK);
        assert_eq!(validator.validate(&"0.0.0.0", &()), OK);
        assert_eq!(validator.validate(&"fe80::223:6cff:fe8a:2e8a", &()), OK);
        assert_eq!(validator.validate(&"::ffff:254.42.16.14", &()), OK);
    }

    #[test]
    fn ip_any_rejects_garbage() {
        let validator = IpValidator { kind: IpKind::Any };
        for bad in ["256.1.1.1", "25.1.1.", "25,1,1,1", "not-an-ip", "2a02::223:6cff :fe8a:2e8a", ""] {
            assert_eq!(
                validator.validate(&bad, &()),
                Err(ValidationError::Ip(IpError { kind: IpKind::Any })),
                "expected `{bad}` to be rejected",
            );
        }
    }

    #[test]
    fn ipv4_accepts_only_v4() {
        let validator = IpValidator { kind: IpKind::V4 };
        assert_eq!(validator.validate(&"1.1.1.1", &()), OK);
        assert_eq!(validator.validate(&"255.0.0.0", &()), OK);
        assert_eq!(
            validator.validate(&"fe80::223:6cff:fe8a:2e8a", &()),
            Err(ValidationError::Ip(IpError { kind: IpKind::V4 })),
        );
        assert_eq!(validator.validate(&"::", &()), Err(ValidationError::Ip(IpError { kind: IpKind::V4 })),);
    }

    #[test]
    fn ipv4_rejects_arabic_digits() {
        let validator = IpValidator { kind: IpKind::V4 };
        assert_eq!(validator.validate(&"٧.2٥.3٣.243", &()), Err(ValidationError::Ip(IpError { kind: IpKind::V4 })),);
    }

    #[test]
    fn ipv6_accepts_only_v6() {
        let validator = IpValidator { kind: IpKind::V6 };
        assert_eq!(validator.validate(&"fe80::223:6cff:fe8a:2e8a", &()), OK);
        assert_eq!(validator.validate(&"2a02::223:6cff:fe8a:2e8a", &()), OK);
        assert_eq!(validator.validate(&"::", &()), OK);
        assert_eq!(validator.validate(&"::a", &()), OK);
        assert_eq!(validator.validate(&"::ffff:254.42.16.14", &()), OK);
        assert_eq!(validator.validate(&"127.0.0.1", &()), Err(ValidationError::Ip(IpError { kind: IpKind::V6 })),);
    }

    #[test]
    fn ipv6_rejects_malformed_inputs() {
        let validator = IpValidator { kind: IpKind::V6 };
        for bad in ["foo", "12345::", "1::2::3::4", "1::zzz", "1:2", "fe80::223: 6cff:fe8a:2e8a"] {
            assert_eq!(
                validator.validate(&bad, &()),
                Err(ValidationError::Ip(IpError { kind: IpKind::V6 })),
                "expected `{bad}` to be rejected",
            );
        }
    }

    #[test]
    fn ip_validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let validator = IpValidator { kind: IpKind::Any };
        let owned: String = "1.1.1.1".to_string();
        assert_eq!(validator.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("fe80::1");
        assert_eq!(validator.validate(&cow, &()), OK);
    }

    #[test]
    fn ip_error_display() {
        assert_eq!(IpError { kind: IpKind::Any }.to_string(), "Ip (expected: any)");
        assert_eq!(IpError { kind: IpKind::V4 }.to_string(), "Ip (expected: v4)");
        assert_eq!(IpError { kind: IpKind::V6 }.to_string(), "Ip (expected: v6)");
    }
}
