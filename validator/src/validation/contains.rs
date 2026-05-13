use std::fmt::Display;

use crate::FieldValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub pattern: String,
    pub case_sensitive: bool,
}

impl<S: AsRef<str>, E: From<MustContainError>, Ctx> FieldValidator<S, E, Ctx> for MustContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let contains = if self.case_sensitive {
            value.as_ref().contains(&self.pattern)
        } else {
            value.as_ref().to_lowercase().contains(&self.pattern.to_lowercase())
        };
        if contains {
            Ok(())
        } else {
            Err(MustContainError { pattern: self.pattern.clone(), case_sensitive: self.case_sensitive }.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub pattern: String,
    pub case_sensitive: bool,
}

impl<S: AsRef<str>, E: From<MustNotContainError>, Ctx> FieldValidator<S, E, Ctx> for MustNotContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let contains = if self.case_sensitive {
            value.as_ref().contains(&self.pattern)
        } else {
            value.as_ref().to_lowercase().contains(&self.pattern.to_lowercase())
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
        let validator = MustContainValidator { pattern: "ell".to_string(), case_sensitive: true };
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"hello".to_string(), &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_passes_when_value_equals_substring() {
        let validator = MustContainValidator { pattern: "hello".to_string(), case_sensitive: true };
        assert_eq!(validator.validate(&"hello", &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_passes_with_empty_substring() {
        let validator = MustContainValidator { pattern: String::new(), case_sensitive: true };
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"", &()), OK);
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_when_substring_absent() {
        let validator = MustContainValidator { pattern: "xyz".to_string(), case_sensitive: true };
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "xyz".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_on_empty_value() {
        let validator = MustContainValidator { pattern: "a".to_string(), case_sensitive: true };
        assert_eq!(
            validator.validate(&"", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "a".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_sensitive_fails_on_case_mismatch() {
        let validator = MustContainValidator { pattern: "Hello".to_string(), case_sensitive: true };
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "Hello".to_string(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_contain_case_insensitive_passes_on_case_mismatch() {
        let validator = MustContainValidator { pattern: "Hello".to_string(), case_sensitive: false };
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"HELLO WORLD", &()), OK);
        assert_eq!(validator.validate(&"say HeLLo!", &()), OK);
    }

    #[test]
    fn test_must_contain_case_insensitive_fails_when_substring_absent() {
        let validator = MustContainValidator { pattern: "xyz".to_string(), case_sensitive: false };
        assert_eq!(
            validator.validate(&"HELLO", &()),
            Err(ValidationError::MustContain(MustContainError { pattern: "xyz".to_string(), case_sensitive: false })),
        );
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_when_substring_absent() {
        let validator = MustNotContainValidator { pattern: "xyz".to_string(), case_sensitive: true };
        assert_eq!(validator.validate(&"hello", &()), OK);
        assert_eq!(validator.validate(&"hello".to_string(), &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_on_empty_value() {
        let validator = MustNotContainValidator { pattern: "a".to_string(), case_sensitive: true };
        assert_eq!(validator.validate(&"", &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_passes_on_case_mismatch() {
        let validator = MustNotContainValidator { pattern: "Hello".to_string(), case_sensitive: true };
        assert_eq!(validator.validate(&"hello", &()), OK);
    }

    #[test]
    fn test_must_not_contain_case_sensitive_fails_when_substring_present() {
        let validator = MustNotContainValidator { pattern: "ell".to_string(), case_sensitive: true };
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
        let validator = MustNotContainValidator { pattern: "hello".to_string(), case_sensitive: true };
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
        let validator = MustNotContainValidator { pattern: String::new(), case_sensitive: true };
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError { pattern: String::new(), case_sensitive: true })),
        );
    }

    #[test]
    fn test_must_not_contain_case_insensitive_fails_on_case_mismatch() {
        let validator = MustNotContainValidator { pattern: "Hello".to_string(), case_sensitive: false };
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
        let validator = MustNotContainValidator { pattern: "xyz".to_string(), case_sensitive: false };
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
