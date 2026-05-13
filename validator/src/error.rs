use thiserror::Error;

use crate::validation::boolean::{MustBeFalseError, MustBeTrueError};


#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {

    #[error("MustBeTrue")]
    MustBeTrue(MustBeTrueError),

    #[error("MustBeFalse")]
    MustBeFalse(MustBeFalseError),

    #[error("MustBeGreater than {min}")]
    MustBeGreater { min: usize },
}