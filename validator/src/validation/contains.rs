use std::fmt::Display;

use thiserror::Error;

use crate::FieldValidator;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct MustContainError {
    pub pattern: String,
    pub case_sensitive: bool,
}

impl Display for MustContainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustContain [{}] (case_sensitive: {})", self.pattern, self.case_sensitive)
    }
}

pub struct MustContainValidator {
    pattern: String,
    /// Lower-cased pattern; pre-computed once at construction when
    /// `case_sensitive == false`. `None` otherwise. Avoids re-running
    /// `pattern.to_lowercase()` (a heap allocation) on every `validate` call.
    pattern_lower: Option<String>,
    case_sensitive: bool,
}

impl MustContainValidator {
    /// Construct a `MustContainValidator`. When `case_sensitive == false`
    /// the pattern's lower-case form is computed once and cached here, so
    /// subsequent `validate` calls only have to lower-case the *value*.
    pub fn new(pattern: impl Into<String>, case_sensitive: bool) -> Self {
        let pattern = pattern.into();
        let pattern_lower = (!case_sensitive).then(|| pattern.to_lowercase());
        Self { pattern, pattern_lower, case_sensitive }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}

impl<S: AsRef<str>, E: From<MustContainError>, Ctx> FieldValidator<S, E, Ctx> for MustContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let contains = match &self.pattern_lower {
            None => value.as_ref().contains(&self.pattern),
            Some(needle) => value.as_ref().to_lowercase().contains(needle.as_str()),
        };
        if contains {
            Ok(())
        } else {
            Err(MustContainError { pattern: self.pattern.clone(), case_sensitive: self.case_sensitive }.into())
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct MustNotContainError {
    pub pattern: String,
    pub case_sensitive: bool,
}

impl Display for MustNotContainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustNotContain [{}] (case_sensitive: {})", self.pattern, self.case_sensitive)
    }
}

pub struct MustNotContainValidator {
    pattern: String,
    /// See [`MustContainValidator::pattern_lower`].
    pattern_lower: Option<String>,
    case_sensitive: bool,
}

impl MustNotContainValidator {
    /// Construct a `MustNotContainValidator`. Mirrors
    /// [`MustContainValidator::new`].
    pub fn new(pattern: impl Into<String>, case_sensitive: bool) -> Self {
        let pattern = pattern.into();
        let pattern_lower = (!case_sensitive).then(|| pattern.to_lowercase());
        Self { pattern, pattern_lower, case_sensitive }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}

impl<S: AsRef<str>, E: From<MustNotContainError>, Ctx> FieldValidator<S, E, Ctx> for MustNotContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let contains = match &self.pattern_lower {
            None => value.as_ref().contains(&self.pattern),
            Some(needle) => value.as_ref().to_lowercase().contains(needle.as_str()),
        };
        if contains {
            Err(MustNotContainError { pattern: self.pattern.clone(), case_sensitive: self.case_sensitive }.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    /// Pinned-type `Ok` for `assert_eq!`: the validator's `validate` impl is
    /// now generic over `E: From<NarrowError>`, so the Ok-side needs a hint
    /// to pick `E = ValidationError` for these tests.
    const OK: Result<(), ValidationError> = Ok(());

    #[test]
    fn test_must_contain_case_sensitive_passes_when_substring_present() {
        let validator = MustContainValidator::new("ell".to_string(), true);
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"hello".to_string(), &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_passes_when_value_equals_substring() {
        let validator = MustContainValidator::new("hello".to_string(), true);
        assert_eq!(validator.validate(&"hello", &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_passes_with_empty_substring() {
        let validator = MustContainValidator::new(String::new(), true);
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"", &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_when_substring_absent() {
        let validator = MustContainValidator::new("xyz".to_string(), true);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "xyz".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_on_empty_value() {
        let validator = MustContainValidator::new("a".to_string(), true);
        assert_eq!(
            validator.validate(&"", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "a".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_on_case_mismatch() {
        let validator = MustContainValidator::new("Hello".to_string(), true);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "Hello".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_insensitive_passes_on_case_mismatch() {
        let validator = MustContainValidator::new("Hello".to_string(), false);
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"HELLO WORLD", &()), OK);
        assert_eq!(validator.validate(&"say HeLLo!", &()), OK);
    }

    #[test]
    fn test_must_contain_case_insensitive_fails_when_substring_absent() {
        let validator = MustContainValidator::new("xyz".to_string(), false);
        assert_eq!(
            validator.validate(&"HELLO", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "xyz".to_string(), case_sensitive: false })),
        );
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_when_substring_absent() {
        let validator = MustNotContainValidator::new("xyz".to_string(), true);
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"hello".to_string(), &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_on_empty_value() {
        let validator = MustNotContainValidator::new("a".to_string(), true);
        assert_eq!(validator.validate(&"", &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_on_case_mismatch() {
        let validator = MustNotContainValidator::new("Hello".to_string(), true);
        assert_eq!(validator.validate(&"hello", &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_fails_when_substring_present() {
        let validator = MustNotContainValidator::new("ell".to_string(), true);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError {
                pattern: "ell".to_string(),
                case_sensitive: true,
            })),
        );
    }

    #[test]
    fn test_must_not_contain_case_sensitive_fails_when_value_equals_substring() {
        let validator = MustNotContainValidator::new("hello".to_string(), true);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError {
                pattern: "hello".to_string(),
                case_sensitive: true,
            })),
        );
    }

    #[test]
    fn test_must_not_contain_case_sensitive_fails_with_empty_substring() {
        let validator = MustNotContainValidator::new(String::new(), true);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError { pattern: String::new(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_not_contain_case_insensitive_fails_on_case_mismatch() {
        let validator = MustNotContainValidator::new("Hello".to_string(), false);
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError {
                pattern: "Hello".to_string(),
                case_sensitive: false,
            })),
        );
        assert_eq!(
            validator.validate(&"say HeLLo!", &()),
            Err(ValidationError::MustNotContain(MustNotContainError {
                pattern: "Hello".to_string(),
                case_sensitive: false,
            })),
        );
    }

    #[test]
    fn test_must_not_contain_case_insensitive_passes_when_substring_absent() {
        let validator = MustNotContainValidator::new("xyz".to_string(), false);
        assert_eq!(validator.validate(&"HELLO", &()), OK);
    }

    #[test]
    fn test_must_contain_error_display() {
        let error = MustContainError { pattern: "foo".to_string(), case_sensitive: true };
        assert_eq!(error.to_string(), "MustContain [foo] (case_sensitive: true)");

        let error = MustContainError { pattern: "foo".to_string(), case_sensitive: false };
        assert_eq!(error.to_string(), "MustContain [foo] (case_sensitive: false)");
    }

    #[test]
    fn test_must_not_contain_error_display() {
        let error = MustNotContainError { pattern: "bar".to_string(), case_sensitive: true };
        assert_eq!(error.to_string(), "MustNotContain [bar] (case_sensitive: true)");

        let error = MustNotContainError { pattern: "bar".to_string(), case_sensitive: false };
        assert_eq!(error.to_string(), "MustNotContain [bar] (case_sensitive: false)");
    }
}
