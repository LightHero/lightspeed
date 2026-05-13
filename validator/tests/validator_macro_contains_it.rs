use std::borrow::Cow;

use lightspeed_validator::contains::{MustContainError, MustNotContainError};
use lightspeed_validator::{Validable, ValidationError};

#[derive(Validable)]
pub struct Email {
    #[validate(contains(pattern = "@"))]
    pub address: String,
    #[validate(not_contains(pattern = "spam"))]
    pub subject: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct CaseFlags {
    #[validate(contains(pattern = "Hello", case_sensitive = false))]
    pub greeting_ci: String,
    #[validate(contains(pattern = "Hello", case_sensitive = true))]
    pub greeting_cs: String,
    #[validate(not_contains(pattern = "Bad", case_sensitive = false))]
    pub body_ci: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(contains(pattern = "ok"))]
    pub note: Cow<'static, str>,
}

#[derive(Validable)]
pub struct StaticStrFields {
    #[validate(contains(pattern = "hi"))]
    pub greeting: &'static str,
}

#[derive(Validable)]
pub struct MultiContainsField {
    #[validate(contains(pattern = "foo"))]
    #[validate(contains(pattern = "bar"))]
    pub via_multiple_attrs: String,
    #[validate(contains(pattern = "foo"), not_contains(pattern = "bar"))]
    pub via_single_attr: String,
}

#[test]
fn contains_validator_passes_when_pattern_present() {
    let v = EmailValidable::new(Email {
        address: "user@example.com".to_string(),
        subject: "hello there".to_string(),
        untouched: String::new(),
    });

    let email = match v.validate() {
        Ok(e) => e,
        Err(_) => panic!("expected Ok"),
    };
    assert_eq!(email.address, "user@example.com");
}

#[test]
fn contains_validator_fails_when_pattern_absent() {
    let v = EmailValidable::new(Email {
        address: "no-at-sign".to_string(),
        subject: "hello there".to_string(),
        untouched: String::new(),
    });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.address.errors(),
        &[ValidationError::MustContain(MustContainError {
            pattern: "@".to_string(),
            case_sensitive: true,
        })],
    );
    assert!(returned.subject.errors().is_empty());
}

#[test]
fn not_contains_validator_passes_when_pattern_absent() {
    let v = EmailValidable::new(Email {
        address: "user@example.com".to_string(),
        subject: "weekly digest".to_string(),
        untouched: String::new(),
    });

    assert!(v.validate().is_ok());
}

#[test]
fn not_contains_validator_fails_when_pattern_present() {
    let v = EmailValidable::new(Email {
        address: "user@example.com".to_string(),
        subject: "this is spam content".to_string(),
        untouched: String::new(),
    });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.address.errors().is_empty());
    assert_eq!(
        returned.subject.errors(),
        &[ValidationError::MustNotContain(MustNotContainError {
            pattern: "spam".to_string(),
            case_sensitive: true,
        })],
    );
}

#[test]
fn case_sensitive_defaults_to_true_when_omitted() {
    let v = EmailValidable::new(Email {
        address: "no at sign".to_string(),
        subject: String::new(),
        untouched: String::new(),
    });
    let validator = &v.address.validators()[0];

    let err = validator.validate(&"HELLO".to_string(), &()).unwrap_err();
    match err {
        ValidationError::MustContain(MustContainError { case_sensitive, .. }) => {
            assert!(case_sensitive, "case_sensitive should default to true");
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn case_insensitive_contains_matches_regardless_of_case() {
    let v = CaseFlagsValidable::new(CaseFlags {
        greeting_ci: "say HELLO there".to_string(),
        greeting_cs: "say Hello there".to_string(),
        body_ci: "all good".to_string(),
    });

    assert!(v.validate().is_ok());
}

#[test]
fn case_sensitive_contains_rejects_case_mismatch() {
    let v = CaseFlagsValidable::new(CaseFlags {
        greeting_ci: "say HELLO there".to_string(),
        greeting_cs: "say hello there".to_string(),
        body_ci: "all good".to_string(),
    });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.greeting_ci.errors().is_empty());
    assert_eq!(
        returned.greeting_cs.errors(),
        &[ValidationError::MustContain(MustContainError {
            pattern: "Hello".to_string(),
            case_sensitive: true,
        })],
    );
}

#[test]
fn case_insensitive_not_contains_rejects_any_casing() {
    let v = CaseFlagsValidable::new(CaseFlags {
        greeting_ci: "HELLO".to_string(),
        greeting_cs: "Hello".to_string(),
        body_ci: "this is BAD news".to_string(),
    });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.body_ci.errors(),
        &[ValidationError::MustNotContain(MustNotContainError {
            pattern: "Bad".to_string(),
            case_sensitive: false,
        })],
    );
}

#[test]
fn contains_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields { note: Cow::Borrowed("this is ok") });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields { note: Cow::Owned("nope".to_string()) });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.note.errors(),
        &[ValidationError::MustContain(MustContainError {
            pattern: "ok".to_string(),
            case_sensitive: true,
        })],
    );
}

#[test]
fn contains_validator_works_on_static_str_field() {
    let v = StaticStrFieldsValidable::new(StaticStrFields { greeting: "hi there" });
    assert!(v.validate().is_ok());

    let v = StaticStrFieldsValidable::new(StaticStrFields { greeting: "bye" });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.greeting.errors(),
        &[ValidationError::MustContain(MustContainError {
            pattern: "hi".to_string(),
            case_sensitive: true,
        })],
    );
}

#[test]
fn macro_attaches_one_validator_per_validate_keyword() {
    let v = EmailValidable::new(Email {
        address: "user@example.com".to_string(),
        subject: "hi".to_string(),
        untouched: String::new(),
    });

    assert_eq!(v.address.validators().len(), 1);
    assert_eq!(v.subject.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}

#[test]
fn macro_accepts_multiple_contains_attributes_on_same_field() {
    let v = MultiContainsFieldValidable::new(MultiContainsField {
        via_multiple_attrs: "foo bar baz".to_string(),
        via_single_attr: "foo and zap".to_string(),
    });

    assert_eq!(v.via_multiple_attrs.validators().len(), 2);
    assert_eq!(v.via_single_attr.validators().len(), 2);
    assert!(v.validate().is_ok());
}

#[test]
fn multiple_contains_validators_emit_each_failure() {
    let v = MultiContainsFieldValidable::new(MultiContainsField {
        via_multiple_attrs: "only foo here".to_string(),
        via_single_attr: "foo and bar".to_string(),
    });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert_eq!(
        returned.via_multiple_attrs.errors(),
        &[ValidationError::MustContain(MustContainError {
            pattern: "bar".to_string(),
            case_sensitive: true,
        })],
        "only the second `contains(bar)` should fail",
    );
    assert_eq!(
        returned.via_single_attr.errors(),
        &[ValidationError::MustNotContain(MustNotContainError {
            pattern: "bar".to_string(),
            case_sensitive: true,
        })],
        "contains(foo) passes, not_contains(bar) fails",
    );
}

#[test]
fn contains_validators_run_in_attribute_order() {
    let v = MultiContainsFieldValidable::new(MultiContainsField {
        via_multiple_attrs: "no matches at all".to_string(),
        via_single_attr: String::new(),
    });

    let mut errs = Vec::new();
    for validator in v.via_multiple_attrs.validators() {
        if let Err(e) = validator.validate(v.via_multiple_attrs.get(), &()) {
            errs.push(e);
        }
    }
    assert_eq!(
        errs,
        vec![
            ValidationError::MustContain(MustContainError {
                pattern: "foo".to_string(),
                case_sensitive: true,
            }),
            ValidationError::MustContain(MustContainError {
                pattern: "bar".to_string(),
                case_sensitive: true,
            }),
        ],
    );
}
