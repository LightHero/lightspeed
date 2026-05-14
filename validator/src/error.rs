use std::collections::HashMap;

use thiserror::Error;

#[cfg(feature = "credit_card")]
use crate::validation::credit_card::CreditCardError;
use crate::{
    contains::{MustContainError, MustNotContainError},
    validation::{
        boolean::{MustBeFalseError, MustBeTrueError},
        email::EmailError,
        fields_match::{FieldsMustMatch, MustMatchField},
        ip::IpError,
        length::LengthError,
        password::PasswordError,
        range::RangeError,
        regex::RegexError,
        url::UrlError,
    },
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NoError {}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error(transparent)]
    MustBeTrue(#[from] MustBeTrueError),

    #[error(transparent)]
    MustBeFalse(#[from] MustBeFalseError),

    #[error(transparent)]
    MustContain(#[from] MustContainError),

    #[error(transparent)]
    MustNotContain(#[from] MustNotContainError),

    #[error(transparent)]
    FieldsMustMatch(#[from] FieldsMustMatch),

    #[error(transparent)]
    MustMatchField(#[from] MustMatchField),

    #[error(transparent)]
    Ip(#[from] IpError),

    #[error(transparent)]
    Url(#[from] UrlError),

    #[error(transparent)]
    Password(#[from] PasswordError),

    #[error(transparent)]
    Range(#[from] RangeError),

    #[error(transparent)]
    Regex(#[from] RegexError),

    #[error(transparent)]
    Length(#[from] LengthError),

    #[error(transparent)]
    Email(#[from] EmailError),

    #[cfg(feature = "credit_card")]
    #[error(transparent)]
    CreditCard(#[from] CreditCardError),

    #[error("ValidationError::Custom {code}: {message} - params: {params:?}")]
    Custom{
        code: String,
        message: String,
        params: HashMap<String, String>,
    },
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let error =
            ValidationError::MustContain(MustContainError { pattern: "hello".to_string(), case_sensitive: true });
        assert_eq!(error.to_string(), "MustContain [hello] (case_sensitive: true)");

        let error = ValidationError::FieldsMustMatch(FieldsMustMatch {
            field_a: "password".to_string(),
            field_b: "password_confirm".to_string(),
        });
        assert_eq!(error.to_string(), "FieldsMustMatch [password, password_confirm]");

        let error = ValidationError::MustMatchField(MustMatchField { field: "password".to_string() });
        assert_eq!(error.to_string(), "MustMatchField [password]");
    }

    #[test]
    fn from_narrow_error_lifts_to_validation_error() {
        let v: ValidationError = MustContainError { pattern: "x".to_string(), case_sensitive: true }.into();
        assert!(matches!(v, ValidationError::MustContain(_)));

        let v: ValidationError = MustMatchField { field: "foo".to_string() }.into();
        assert!(matches!(v, ValidationError::MustMatchField(_)));
    }
}
