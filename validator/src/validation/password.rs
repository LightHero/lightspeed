use std::fmt::Display;

use crate::FieldValidator;

/// Recommended special-character set used when [`PasswordValidator`]'s
/// `special_chars` is `Some` but no custom list was provided. Matches the
/// printable-ASCII non-alphanumeric, non-space set commonly recommended for
/// password policies.
pub const DEFAULT_SPECIAL_CHARS: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?', '@', '[',
    '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordViolation {
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialChar,
    HasTrailingWhitespace,
}

impl Display for PasswordViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUppercase => write!(f, "missing_uppercase"),
            Self::MissingLowercase => write!(f, "missing_lowercase"),
            Self::MissingNumber => write!(f, "missing_number"),
            Self::MissingSpecialChar => write!(f, "missing_special_char"),
            Self::HasTrailingWhitespace => write!(f, "has_trailing_whitespace"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordError {
    pub violations: Vec<PasswordViolation>,
}

impl Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password [")?;
        for (i, v) in self.violations.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            Display::fmt(v, f)?;
        }
        write!(f, "]")
    }
}

/// Validates a string against a configurable password policy. All
/// character-class checks use ASCII (`is_ascii_uppercase` /
/// `is_ascii_lowercase` / `is_ascii_digit`); the special-character check
/// uses the configured `special_chars` list verbatim. The trailing-whitespace
/// check is Unicode-aware (`char::is_whitespace`).
pub struct PasswordValidator {
    /// Require at least one ASCII uppercase letter.
    pub upper: bool,
    /// Require at least one ASCII lowercase letter.
    pub lower: bool,
    /// Require at least one ASCII digit.
    pub number: bool,
    /// `Some(list)` requires at least one char from `list`. `None` disables
    /// the special-character check entirely.
    pub special_chars: Option<Vec<char>>,
    /// When `false`, the password must not end in a whitespace character.
    /// Set to `true` to allow trailing whitespace.
    pub trailing_whitespaces: bool,
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self {
            upper: true,
            lower: true,
            number: true,
            special_chars: Some(DEFAULT_SPECIAL_CHARS.to_vec()),
            trailing_whitespaces: false,
        }
    }
}

impl<S: AsRef<str>, E: From<PasswordError>, Ctx> FieldValidator<S, E, Ctx> for PasswordValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let s = value.as_ref();
        let mut violations = Vec::new();

        if self.upper && !s.chars().any(|c| c.is_ascii_uppercase()) {
            violations.push(PasswordViolation::MissingUppercase);
        }
        if self.lower && !s.chars().any(|c| c.is_ascii_lowercase()) {
            violations.push(PasswordViolation::MissingLowercase);
        }
        if self.number && !s.chars().any(|c| c.is_ascii_digit()) {
            violations.push(PasswordViolation::MissingNumber);
        }
        if let Some(allowed) = &self.special_chars
            && !s.chars().any(|c| allowed.contains(&c))
        {
            violations.push(PasswordViolation::MissingSpecialChar);
        }
        if !self.trailing_whitespaces && s.chars().next_back().is_some_and(char::is_whitespace) {
            violations.push(PasswordViolation::HasTrailingWhitespace);
        }

        if violations.is_empty() { Ok(()) } else { Err(PasswordError { violations }.into()) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    fn err(violations: &[PasswordViolation]) -> Result<(), ValidationError> {
        Err(ValidationError::Password(PasswordError { violations: violations.to_vec() }))
    }

    #[test]
    fn default_config_accepts_strong_password() {
        let v = PasswordValidator::default();
        assert_eq!(v.validate(&"Aa1!xyz", &()), OK);
        assert_eq!(v.validate(&"P@ssw0rd", &()), OK);
    }

    #[test]
    fn default_config_lists_every_missing_class() {
        let v = PasswordValidator::default();
        assert_eq!(
            v.validate(&"aaaa", &()),
            err(&[
                PasswordViolation::MissingUppercase,
                PasswordViolation::MissingNumber,
                PasswordViolation::MissingSpecialChar,
            ]),
        );
    }

    #[test]
    fn default_config_rejects_trailing_whitespace() {
        let v = PasswordValidator::default();
        assert_eq!(v.validate(&"Aa1!xyz ", &()), err(&[PasswordViolation::HasTrailingWhitespace]),);
        // Tab also counts.
        assert_eq!(v.validate(&"Aa1!xyz\t", &()), err(&[PasswordViolation::HasTrailingWhitespace]),);
    }

    #[test]
    fn trailing_whitespaces_true_allows_trailing_space() {
        let v = PasswordValidator { trailing_whitespaces: true, ..PasswordValidator::default() };
        assert_eq!(v.validate(&"Aa1!xyz ", &()), OK);
    }

    #[test]
    fn disabling_upper_allows_lowercase_only_passwords() {
        let v = PasswordValidator {
            upper: false,
            lower: true,
            number: false,
            special_chars: None,
            trailing_whitespaces: false,
        };
        assert_eq!(v.validate(&"hello", &()), OK);
    }

    #[test]
    fn custom_special_chars_list_restricts_accepted_set() {
        let v = PasswordValidator {
            upper: false,
            lower: false,
            number: false,
            special_chars: Some(vec!['*', '$']),
            trailing_whitespaces: false,
        };
        assert_eq!(v.validate(&"abc*def", &()), OK);
        assert_eq!(v.validate(&"abc$def", &()), OK);
        // `!` is in DEFAULT_SPECIAL_CHARS but not in this custom list.
        assert_eq!(v.validate(&"abc!def", &()), err(&[PasswordViolation::MissingSpecialChar]),);
    }

    #[test]
    fn special_chars_none_disables_the_check() {
        let v = PasswordValidator {
            upper: false,
            lower: false,
            number: false,
            special_chars: None,
            trailing_whitespaces: false,
        };
        assert_eq!(v.validate(&"plain", &()), OK);
    }

    #[test]
    fn empty_string_fails_all_required_classes() {
        let v = PasswordValidator::default();
        assert_eq!(
            v.validate(&"", &()),
            err(&[
                PasswordViolation::MissingUppercase,
                PasswordViolation::MissingLowercase,
                PasswordViolation::MissingNumber,
                PasswordViolation::MissingSpecialChar,
            ]),
        );
    }

    #[test]
    fn password_validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let v = PasswordValidator::default();
        let owned: String = "Aa1!xyz".to_string();
        assert_eq!(v.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("Aa1!xyz");
        assert_eq!(v.validate(&cow, &()), OK);
    }

    #[test]
    fn password_error_display() {
        let e =
            PasswordError { violations: vec![PasswordViolation::MissingUppercase, PasswordViolation::MissingNumber] };
        assert_eq!(e.to_string(), "Password [missing_uppercase, missing_number]");
    }
}
