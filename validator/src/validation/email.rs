use std::fmt::Display;

use thiserror::Error;

use crate::FieldValidator;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct EmailError;

impl Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email")
    }
}

/// Validates that a string-compatible value parses as an email address via
/// the [`email_address`](https://docs.rs/email_address) crate (RFC 5321
/// / 5322 compliant). The validator only checks shape — it does not perform
/// DNS lookups, mailbox-reachability checks, or accept-list filtering.
pub struct EmailValidator;

impl<S: AsRef<str>, E: From<EmailError>, Ctx> FieldValidator<S, E, Ctx> for EmailValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        if ::email_address::EmailAddress::is_valid(value.as_ref()) { Ok(()) } else { Err(EmailError.into()) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    #[test]
    fn accepts_well_formed_emails() {
        for ok in ["user@example.com", "u.s.e.r+tag@sub.example.com", "first.last@example.co.uk", "x@y.z"] {
            assert_eq!(EmailValidator.validate(&ok, &()), OK, "expected `{ok}` accepted");
        }
    }

    #[test]
    fn rejects_garbage_and_partial_addresses() {
        for bad in ["", "not-an-email", "user@", "@example.com", "user@@example.com", "user.example.com"] {
            assert_eq!(
                EmailValidator.validate(&bad, &()),
                Err(ValidationError::Email(EmailError)),
                "expected `{bad}` rejected",
            );
        }
    }

    #[test]
    fn validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let owned: String = "user@example.com".to_string();
        assert_eq!(EmailValidator.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("user@example.com");
        assert_eq!(EmailValidator.validate(&cow, &()), OK);
    }

    #[test]
    fn email_error_display() {
        assert_eq!(EmailError.to_string(), "Email");
    }
}
