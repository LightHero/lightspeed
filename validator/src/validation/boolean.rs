use std::fmt::Display;

use thiserror::Error;

use crate::FieldValidator;

/// Must be true
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct MustBeTrueError;

impl Display for MustBeTrueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustBeTrue")
    }
}

/// Must be false
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct MustBeFalseError;

impl Display for MustBeFalseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustBeFalse")
    }
}

/// Program-wide stateless instance of [`MustBeTrueValidator`]. Used by the
/// macro to avoid a per-validator `Box::new` heap allocation.
pub static MUST_BE_TRUE_VALIDATOR: MustBeTrueValidator = MustBeTrueValidator;

/// Program-wide stateless instance of [`MustBeFalseValidator`].
pub static MUST_BE_FALSE_VALIDATOR: MustBeFalseValidator = MustBeFalseValidator;

/// validate that a value is true
pub struct MustBeTrueValidator;

impl<E: From<MustBeTrueError>, Ctx> FieldValidator<bool, E, Ctx> for MustBeTrueValidator {
    fn validate(&self, value: &bool, _context: &Ctx) -> Result<(), E> {
        if *value { Ok(()) } else { Err(MustBeTrueError.into()) }
    }
}

/// validate that a value is false
pub struct MustBeFalseValidator;

impl<E: From<MustBeFalseError>, Ctx> FieldValidator<bool, E, Ctx> for MustBeFalseValidator {
    fn validate(&self, value: &bool, _context: &Ctx) -> Result<(), E> {
        if *value { Err(MustBeFalseError.into()) } else { Ok(()) }
    }
}

#[cfg(test)]
mod test {

    use crate::ValidationError;

    use super::*;

    #[test]
    fn test_must_be_true() {
        assert_eq!(
            <MustBeTrueValidator as FieldValidator<bool, ValidationError, ()>>::validate(
                &MustBeTrueValidator,
                &false,
                &()
            ),
            Err(ValidationError::MustBeTrue(MustBeTrueError))
        );
        assert_eq!(
            <MustBeTrueValidator as FieldValidator<bool, ValidationError, ()>>::validate(
                &MustBeTrueValidator,
                &true,
                &()
            ),
            Ok(())
        );
    }

    #[test]
    fn test_must_be_false() {
        assert_eq!(
            <MustBeFalseValidator as FieldValidator<bool, ValidationError, ()>>::validate(
                &MustBeFalseValidator,
                &true,
                &()
            ),
            Err(ValidationError::MustBeFalse(MustBeFalseError))
        );
        assert_eq!(
            <MustBeFalseValidator as FieldValidator<bool, ValidationError, ()>>::validate(
                &MustBeFalseValidator,
                &false,
                &()
            ),
            Ok(())
        );
    }
}
