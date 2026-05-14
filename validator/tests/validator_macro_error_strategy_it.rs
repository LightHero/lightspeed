use std::borrow::Cow;

use lightspeed_validator::contains::MustContainError;
use lightspeed_validator::length::LengthError;
use lightspeed_validator::{NoError, Validable, ValidableType, ValidationError};

// ---- shared (the default) ------------------------------------------------

#[derive(Validable)]
pub struct SignupShared {
    #[validate(contains(pattern = "@"))]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub display_name: String,
}

#[test]
fn shared_is_the_default_and_uses_validation_error_everywhere() {
    fn assert_types(v: &SignupSharedValidable) {
        // Every field — even the one with no validators — uses `ValidationError`.
        let _: &ValidableType<String, ValidationError, ()> = &v.email;
        let _: &ValidableType<String, ValidationError, ()> = &v.password;
        let _: &ValidableType<String, ValidationError, ()> = &v.display_name;
    }
    let _ = assert_types;

    let v = SignupSharedValidable::new(SignupShared {
        email: "no-at-sign".to_string(),
        password: "short".to_string(),
        display_name: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    // Both fields' errors are `ValidationError`, so they can be put into the
    // same `Vec<ValidationError>` and matched against the wide enum.
    let all: Vec<&ValidationError> =
        returned.email.errors().iter().chain(returned.password.errors().iter()).collect();
    assert_eq!(all.len(), 2);
    assert!(matches!(all[0], ValidationError::MustContain(_)));
    assert!(matches!(all[1], ValidationError::Length(_)));
}

// ---- tailored ------------------------------------------------------------

#[derive(Validable)]
#[validate(errors(tailored))]
pub struct SignupTailored {
    #[validate(contains(pattern = "@"))]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub display_name: String,
}

#[test]
fn tailored_generates_per_field_enums_and_no_error_for_empty_fields() {
    fn assert_types(v: &SignupTailoredValidable) {
        let _: &ValidableType<String, SignupTailoredEmailFieldError, ()> = &v.email;
        let _: &ValidableType<String, SignupTailoredPasswordFieldError, ()> = &v.password;
        // No validators → `NoError`, errors vec is statically empty.
        let _: &ValidableType<String, NoError, ()> = &v.display_name;
    }
    let _ = assert_types;

    let v = SignupTailoredValidable::new(SignupTailored {
        email: "no-at-sign".to_string(),
        password: "short".to_string(),
        display_name: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    // Exhaustive match against the field's own enum — no other variants
    // exist, so this is the entire failure surface for `email`.
    for err in returned.email.errors() {
        match err {
            SignupTailoredEmailFieldError::MustContain(MustContainError { pattern, case_sensitive }) => {
                assert_eq!(*pattern, "@");
                assert!(*case_sensitive);
            }
        }
    }
    for err in returned.password.errors() {
        match err {
            SignupTailoredPasswordFieldError::Length(LengthError { actual, min, .. }) => {
                assert_eq!(*actual, 5);
                assert_eq!(*min, Some(8));
            }
        }
    }
    // `display_name` has `E = NoError`; the errors slice can never hold
    // anything, so it is statically always empty.
    let _: &[NoError] = returned.display_name.errors();
    assert!(returned.display_name.errors().is_empty());
}


/// A user-defined error type for the `custom` strategy. Must implement
/// `From<NarrowError>` for every narrow error emitted by any validator on
/// the struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignupCustomError {
    BadEmail(MustContainError),
    BadLength(LengthError),
}

impl From<MustContainError> for SignupCustomError {
    fn from(e: MustContainError) -> Self {
        SignupCustomError::BadEmail(e)
    }
}

impl From<LengthError> for SignupCustomError {
    fn from(e: LengthError) -> Self {
        SignupCustomError::BadLength(e)
    }
}

#[derive(Validable)]
#[validate(errors(custom = SignupCustomError))]
pub struct SignupCustom {
    #[validate(contains(pattern = "@"))]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub display_name: String,
}

#[test]
fn custom_strategy_uses_user_provided_error_for_every_field() {
    fn assert_types(v: &SignupCustomValidable) {
        let _: &ValidableType<String, SignupCustomError, ()> = &v.email;
        let _: &ValidableType<String, SignupCustomError, ()> = &v.password;
        // No validators on this field → still `SignupCustomError`, vec just stays empty.
        let _: &ValidableType<String, SignupCustomError, ()> = &v.display_name;
    }
    let _ = assert_types;

    let v = SignupCustomValidable::new(SignupCustom {
        email: "no-at-sign".to_string(),
        password: "short".to_string(),
        display_name: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert!(matches!(returned.email.errors(), [SignupCustomError::BadEmail(_)]));
    assert!(matches!(returned.password.errors(), [SignupCustomError::BadLength(_)]));
    assert!(returned.display_name.errors().is_empty());

    // The whole struct's failures can flow into a single homogeneous vec:
    let mut all: Vec<SignupCustomError> = Vec::new();
    all.extend(returned.email.errors().iter().cloned());
    all.extend(returned.password.errors().iter().cloned());
    assert_eq!(all.len(), 2);
}


/// Test that the custom strategy still composes with string-compatible
/// non-`String` field types.
#[derive(Validable)]
#[validate(errors(custom = SignupCustomError))]
pub struct CowFieldWithCustomErrors {
    #[validate(contains(pattern = "ok"))]
    pub note: Cow<'static, str>,
}

#[test]
fn custom_strategy_works_with_cow_field_types() {
    let v = CowFieldWithCustomErrorsValidable::new(CowFieldWithCustomErrors {
        note: Cow::Borrowed("this is ok"),
    });
    assert!(v.validate().is_ok());

    let v = CowFieldWithCustomErrorsValidable::new(CowFieldWithCustomErrors {
        note: Cow::Borrowed("nope"),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(matches!(returned.note.errors(), [SignupCustomError::BadEmail(_)]));
}
