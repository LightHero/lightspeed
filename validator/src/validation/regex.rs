use std::fmt::Display;

use thiserror::Error;

use crate::FieldValidator;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct RegexError {
    pub pattern: String,
}

impl Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Regex [pattern: {}]", self.pattern)
    }
}

/// Validates that a string-compatible value matches the wrapped regex via
/// [`Regex::is_match`](::regex::Regex::is_match). The regex is held as a
/// `&'static Regex` so the validator never recompiles it; callers are
/// expected to back the static reference with a `OnceLock<Regex>` or
/// `LazyLock<Regex>` (or any other mechanism that yields a `'static`
/// reference).
pub struct RegexValidator {
    pub regex: &'static ::regex::Regex,
}

impl<S: AsRef<str>, E: From<RegexError>, Ctx> FieldValidator<S, E, Ctx> for RegexValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        if self.regex.is_match(value.as_ref()) {
            Ok(())
        } else {
            Err(RegexError { pattern: self.regex.as_str().to_string() }.into())
        }
    }
}

#[cfg(test)]
mod test {

    use std::sync::LazyLock;

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    static EMAIL_RE: LazyLock<::regex::Regex> =
        LazyLock::new(|| ::regex::Regex::new(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap());

    static DIGIT_RE: LazyLock<::regex::Regex> = LazyLock::new(|| ::regex::Regex::new(r"^\d+$").unwrap());

    #[test]
    fn accepts_matching_strings() {
        let v = RegexValidator { regex: &EMAIL_RE };
        assert_eq!(v.validate(&"user@example.com", &()), OK);
        assert_eq!(v.validate(&"u.s.e.r+tag@sub.example.com", &()), OK);
    }

    #[test]
    fn rejects_non_matching_strings_with_pattern_in_error() {
        let v = RegexValidator { regex: &EMAIL_RE };
        assert_eq!(
            v.validate(&"not-an-email", &()),
            Err(ValidationError::Regex(RegexError { pattern: r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$".to_string() })),
        );
    }

    #[test]
    fn validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let v = RegexValidator { regex: &DIGIT_RE };
        let owned: String = "12345".to_string();
        assert_eq!(v.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("0");
        assert_eq!(v.validate(&cow, &()), OK);
        assert_eq!(v.validate(&"abc", &()), Err(ValidationError::Regex(RegexError { pattern: r"^\d+$".to_string() })),);
    }

    #[test]
    fn regex_error_display() {
        let e = RegexError { pattern: r"^\d+$".to_string() };
        assert_eq!(e.to_string(), r"Regex [pattern: ^\d+$]");
    }
}
