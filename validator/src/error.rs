use thiserror::Error;

use crate::{contains::{MustContainError, MustNotContainError}, validation::boolean::{MustBeFalseError, MustBeTrueError}};


#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {

    #[error("{0}")]
    MustBeTrue(MustBeTrueError),

    #[error("{0}")]
    MustBeFalse(MustBeFalseError),

    #[error("{0}")]
    MustContain(MustContainError),

    #[error("{0}")]
    MustNotContain(MustNotContainError),

    #[error("MustBeGreater than {min}")]
    MustBeGreater { min: usize },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let error = ValidationError::MustBeGreater { min: 5 };
        assert_eq!(error.to_string(), "MustBeGreater than 5");

        let error = ValidationError::MustContain(MustContainError("hello".to_string()));
        assert_eq!(error.to_string(), "MustContain [hello]");
    }
}