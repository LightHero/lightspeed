use std::fmt::Display;

use crate::{FieldValidator, ValidationError};

#[derive(Debug, PartialEq, Eq)]
pub struct MustContainError(pub String);

impl Display for MustContainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustContain [{}]", self.0)
    }
}

pub struct MustContainValidator(pub String);

impl <S: AsRef<str>, Ctx> FieldValidator<S, ValidationError, Ctx> for MustContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), ValidationError> {
        if value.as_ref().contains(&self.0) {
            Ok(())
        } else {
            Err(ValidationError::MustContain(MustContainError(self.0.clone())))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MustNotContainError(String);

impl Display for MustNotContainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustNotContain [{}]", self.0)
    }
}

pub struct MustNotContainValidator(String);

impl <S: AsRef<str>, Ctx> FieldValidator<S, ValidationError, Ctx> for MustNotContainValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), ValidationError> {
        if value.as_ref().contains(&self.0) {
            Err(ValidationError::MustNotContain(MustNotContainError(self.0.clone())))
        } else {
            Ok(())
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_must_contain_passes_when_substring_present() {
        let validator = MustContainValidator("ell".to_string());
        assert_eq!(validator.validate(&"hello", &()), Ok(()));
        assert_eq!(validator.validate(&"hello".to_string(), &()), Ok(()));
    }

    #[test]
    fn test_must_contain_passes_when_value_equals_substring() {
        let validator = MustContainValidator("hello".to_string());
        assert_eq!(validator.validate(&"hello", &()), Ok(()));
    }

    #[test]
    fn test_must_contain_passes_with_empty_substring() {
        let validator = MustContainValidator(String::new());
        assert_eq!(validator.validate(&"hello", &()), Ok(()));
        assert_eq!(validator.validate(&"", &()), Ok(()));
    }

    #[test]
    fn test_must_contain_fails_when_substring_absent() {
        let validator = MustContainValidator("xyz".to_string());
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError("xyz".to_string()))),
        );
    }

    #[test]
    fn test_must_contain_fails_on_empty_value() {
        let validator = MustContainValidator("a".to_string());
        assert_eq!(
            validator.validate(&"", &()),
            Err(ValidationError::MustContain(MustContainError("a".to_string()))),
        );
    }

    #[test]
    fn test_must_contain_is_case_sensitive() {
        let validator = MustContainValidator("Hello".to_string());
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustContain(MustContainError("Hello".to_string()))),
        );
    }

    #[test]
    fn test_must_not_contain_passes_when_substring_absent() {
        let validator = MustNotContainValidator("xyz".to_string());
        assert_eq!(validator.validate(&"hello", &()), Ok(()));
        assert_eq!(validator.validate(&"hello".to_string(), &()), Ok(()));
    }

    #[test]
    fn test_must_not_contain_passes_on_empty_value() {
        let validator = MustNotContainValidator("a".to_string());
        assert_eq!(validator.validate(&"", &()), Ok(()));
    }

    #[test]
    fn test_must_not_contain_is_case_sensitive() {
        let validator = MustNotContainValidator("Hello".to_string());
        assert_eq!(validator.validate(&"hello", &()), Ok(()));
    }

    #[test]
    fn test_must_not_contain_fails_when_substring_present() {
        let validator = MustNotContainValidator("ell".to_string());
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError("ell".to_string()))),
        );
    }

    #[test]
    fn test_must_not_contain_fails_when_value_equals_substring() {
        let validator = MustNotContainValidator("hello".to_string());
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError("hello".to_string()))),
        );
    }

    #[test]
    fn test_must_not_contain_fails_with_empty_substring() {
        let validator = MustNotContainValidator(String::new());
        assert_eq!(
            validator.validate(&"hello", &()),
            Err(ValidationError::MustNotContain(MustNotContainError(String::new()))),
        );
    }

    #[test]
    fn test_must_contain_error_display() {
        let error = MustContainError("foo".to_string());
        assert_eq!(error.to_string(), "MustContain [foo]");
    }

    #[test]
    fn test_must_not_contain_error_display() {
        let error = MustNotContainError("bar".to_string());
        assert_eq!(error.to_string(), "MustNotContain [bar]");
    }
}
