use thiserror::Error;

use crate::{
    contains::{MustContainError, MustNotContainError},
    validation::{
        boolean::{MustBeFalseError, MustBeTrueError},
        fields_match::{FieldsMustMatch, MustMatchField},
        ip::IpError,
        password::PasswordError,
        range::RangeError,
        regex::RegexError,
        url::UrlError,
    },
};
#[cfg(feature = "credit_card")]
use crate::validation::credit_card::CreditCardError;


#[derive(Debug, Error, Clone, PartialEq, Eq)]
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

    #[error("{0}")]
    FieldsMustMatch(FieldsMustMatch),

    #[error("{0}")]
    MustMatchField(MustMatchField),

    #[error("{0}")]
    Ip(IpError),

    #[error("{0}")]
    Url(UrlError),

    #[error("{0}")]
    Password(PasswordError),

    #[error("{0}")]
    Range(RangeError),

    #[error("{0}")]
    Regex(RegexError),

    #[cfg(feature = "credit_card")]
    #[error("{0}")]
    CreditCard(CreditCardError),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let error = ValidationError::MustBeGreater { min: 5 };
        assert_eq!(error.to_string(), "MustBeGreater than 5");

        let error = ValidationError::MustContain(MustContainError {
            pattern: "hello".to_string(),
            case_sensitive: true,
        });
        assert_eq!(error.to_string(), "MustContain [hello] (case_sensitive: true)");

        let error = ValidationError::FieldsMustMatch(FieldsMustMatch {
            field_a: "password".to_string(),
            field_b: "password_confirm".to_string(),
        });
        assert_eq!(error.to_string(), "FieldsMustMatch [password, password_confirm]");

        let error =
            ValidationError::MustMatchField(MustMatchField { field: "password".to_string() });
        assert_eq!(error.to_string(), "MustMatchField [password]");
    }
}
