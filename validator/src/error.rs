use thiserror::Error;

use crate::{
    contains::{MustContainError, MustNotContainError},
    validation::{
        boolean::{MustBeFalseError, MustBeTrueError},
        fields_match::{FieldsMustMatch, MustMatchField},
        ip::IpError,
        length::LengthError,
        password::PasswordError,
        range::RangeError,
        regex::RegexError,
        url::UrlError,
    },
};
#[cfg(feature = "credit_card")]
use crate::validation::credit_card::CreditCardError;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NoError {
}

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

    #[error("{0}")]
    Length(LengthError),

    #[cfg(feature = "credit_card")]
    #[error("{0}")]
    CreditCard(CreditCardError),
}

// `From<NarrowError> for ValidationError` impls. These let every built-in
// `FieldValidator` impl be generic over `E: From<TheirNarrowError>` while
// still working when the chosen `E` is `ValidationError`. They're also the
// fallback used by macro-generated per-field error enums when converting
// upward to `ValidationError`.
macro_rules! impl_from_for_validation_error {
    ($variant:ident, $ty:path) => {
        impl From<$ty> for ValidationError {
            fn from(e: $ty) -> Self {
                ValidationError::$variant(e)
            }
        }
    };
}

impl_from_for_validation_error!(MustBeTrue, MustBeTrueError);
impl_from_for_validation_error!(MustBeFalse, MustBeFalseError);
impl_from_for_validation_error!(MustContain, MustContainError);
impl_from_for_validation_error!(MustNotContain, MustNotContainError);
impl_from_for_validation_error!(FieldsMustMatch, FieldsMustMatch);
impl_from_for_validation_error!(MustMatchField, MustMatchField);
impl_from_for_validation_error!(Ip, IpError);
impl_from_for_validation_error!(Url, UrlError);
impl_from_for_validation_error!(Password, PasswordError);
impl_from_for_validation_error!(Range, RangeError);
impl_from_for_validation_error!(Regex, RegexError);
impl_from_for_validation_error!(Length, LengthError);
#[cfg(feature = "credit_card")]
impl_from_for_validation_error!(CreditCard, CreditCardError);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
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

    #[test]
    fn from_narrow_error_lifts_to_validation_error() {
        let v: ValidationError = MustContainError {
            pattern: "x".to_string(),
            case_sensitive: true,
        }
        .into();
        assert!(matches!(v, ValidationError::MustContain(_)));

        let v: ValidationError = MustMatchField { field: "foo".to_string() }.into();
        assert!(matches!(v, ValidationError::MustMatchField(_)));
    }
}
