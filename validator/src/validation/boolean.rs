use std::fmt::Display;

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
pub fn must_be_true(value: bool) -> Result<(), MustBeTrueError> {
    if value {
        Err(MustBeTrueError)
    } else {
        Ok(())
    }
}

/// validate that a value is false
pub fn must_be_false(value: bool) -> Result<(), MustBeFalseError> {
    if value {
        Ok(())
    } else {
        Err(MustBeFalseError)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_must_be_true() {
        assert_eq!(must_be_true(true), Err(MustBeTrueError));
        assert_eq!(must_be_true(false), Ok(()));
    }

    #[test]
    fn test_must_be_false() {
        assert_eq!(must_be_false(true), Ok(()));
        assert_eq!(must_be_false(false), Err(MustBeFalseError));
    }

}