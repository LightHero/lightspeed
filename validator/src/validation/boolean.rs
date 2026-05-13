use std::fmt::Display;

use crate::{FieldValidator, ValidationError};

/// Must be true
#[derive(Debug, PartialEq, Eq)]
pub struct MustBeTrueError;

impl Display for MustBeTrueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustBeTrueError")
    }
}

/// Must be false
#[derive(Debug, PartialEq, Eq)]
pub struct MustBeFalseError;

impl Display for MustBeFalseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustBeFalseError")
    }
}

/// validate that a value is true
pub struct MustBeTrueValidator;

impl FieldValidator<bool, ValidationError, ()> for MustBeTrueValidator {
    fn validate(&self, value: &bool, _context: &()) -> Result<(), ValidationError> {
        if *value {
            Ok(())
        } else {
            Err(ValidationError::MustBeTrue(MustBeTrueError))
        }
    }
}

/// validate that a value is false
pub struct MustBeFalseValidator;

impl FieldValidator<bool, ValidationError, ()> for MustBeFalseValidator {
    fn validate(&self, value: &bool, _context: &()) -> Result<(), ValidationError> {
        if *value {
            Err(ValidationError::MustBeFalse(MustBeFalseError))
        } else {
            Ok(())
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_must_be_true() {
        assert_eq!(MustBeTrueValidator.validate(&false, &()), Err(ValidationError::MustBeTrue(MustBeTrueError)));
        assert_eq!(MustBeTrueValidator.validate(&true, &()), Ok(()));
    }

    #[test]
    fn test_must_be_false() {
        assert_eq!(MustBeFalseValidator.validate(&true, &()), Err(ValidationError::MustBeFalse(MustBeFalseError)));
        assert_eq!(MustBeFalseValidator.validate(&false, &()), Ok(()));
    }

}