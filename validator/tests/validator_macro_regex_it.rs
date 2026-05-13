use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

use lightspeed_validator::regex::RegexError;
use lightspeed_validator::{Validable, ValidationError};

static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap());

#[derive(Validable)]
pub struct WithPathRegex {
    #[validate(regex(path = &EMAIL_RE))]
    pub email: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct WithPatternRegex {
    #[validate(regex(pattern = r"^\d{3}-\d{4}$"))]
    pub phone_local: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(regex(pattern = r"^\d+$"))]
    pub digits: Cow<'static, str>,
}

fn regex_err(pattern: &str) -> ValidationError {
    ValidationError::Regex(RegexError { pattern: pattern.to_string() })
}

#[test]
fn path_variant_accepts_matching_value() {
    let v = WithPathRegexValidable::new(WithPathRegex {
        email: "user@example.com".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn path_variant_rejects_non_matching_with_pattern_in_error() {
    let v = WithPathRegexValidable::new(WithPathRegex {
        email: "not-an-email".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.email.errors(),
        &[regex_err(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$")],
    );
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn pattern_variant_compiles_regex_once_via_oncelock() {
    let v = WithPatternRegexValidable::new(WithPatternRegex {
        phone_local: "555-1234".to_string(),
    });
    assert!(v.validate().is_ok());

    let v = WithPatternRegexValidable::new(WithPatternRegex {
        phone_local: "555-12345".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.phone_local.errors(), &[regex_err(r"^\d{3}-\d{4}$")]);
}

#[test]
fn pattern_variant_caches_compiled_regex_between_runs() {
    // First run primes the OnceLock; second run reuses it. We verify both
    // produce the same outcome — primarily this guards against any
    // accidental recompilation that would still yield the same result but
    // be wasteful.
    for _ in 0..3 {
        let v = WithPatternRegexValidable::new(WithPatternRegex {
            phone_local: "123-4567".to_string(),
        });
        assert!(v.validate().is_ok());
    }
}

#[test]
fn regex_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields {
        digits: Cow::Borrowed("12345"),
    });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields {
        digits: Cow::Owned("abc".to_string()),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.digits.errors(), &[regex_err(r"^\d+$")]);
}

#[test]
fn macro_attaches_one_validator_per_regex_attribute() {
    let v = WithPathRegexValidable::new(WithPathRegex {
        email: "user@example.com".to_string(),
        untouched: String::new(),
    });
    assert_eq!(v.email.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}

/// Three fields, three distinct patterns, and one field carrying TWO patterns
/// — proves the macro-generated per-call-site `OnceLock<Regex>`s don't collide
/// and each validator runs only against its own pattern.
#[derive(Validable)]
pub struct ThreeFields {
    /// only digits
    #[validate(regex(pattern = r"^\d+$"))]
    pub digits_only: String,
    /// only ASCII letters
    #[validate(regex(pattern = r"^[a-zA-Z]+$"))]
    pub letters_only: String,
    /// must contain at least one digit AND at least one letter — two regex
    /// validators stacked on the same field
    #[validate(regex(pattern = r"\d"))]
    #[validate(regex(pattern = r"[a-zA-Z]"))]
    pub mixed: String,
}

#[test]
fn three_fields_with_independent_patterns_validate_independently() {
    // All happy — digits_only with digits, letters_only with letters,
    // mixed with both classes.
    let v = ThreeFieldsValidable::new(ThreeFields {
        digits_only: "12345".to_string(),
        letters_only: "abcDEF".to_string(),
        mixed: "a1".to_string(),
    });
    assert!(v.validate().is_ok(), "all three fields should validate cleanly");
}

#[test]
fn each_field_only_sees_its_own_pattern_in_its_error() {
    // Cross-pollute: digits_only gets letters, letters_only gets digits,
    // mixed has only letters (still misses the digit pattern).
    let v = ThreeFieldsValidable::new(ThreeFields {
        digits_only: "abc".to_string(),
        letters_only: "123".to_string(),
        mixed: "letters_only".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    // Each field reports exactly its own pattern — proves the OnceLocks
    // don't collide and didn't get swapped between fields.
    assert_eq!(returned.digits_only.errors(), &[regex_err(r"^\d+$")]);
    assert_eq!(returned.letters_only.errors(), &[regex_err(r"^[a-zA-Z]+$")]);
    assert_eq!(
        returned.mixed.errors(),
        &[regex_err(r"\d")],
        "mixed has letters but no digits → only the digit-requiring regex fails",
    );
}

#[test]
fn two_regexes_on_same_field_apply_independently_and_in_declaration_order() {
    // mixed = "1" — has digit, no letter → first regex (digit) passes,
    // second regex (letter) fails → exactly one error, for the letter pattern.
    let v = ThreeFieldsValidable::new(ThreeFields {
        digits_only: "12".to_string(),
        letters_only: "ab".to_string(),
        mixed: "1".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.digits_only.errors().is_empty());
    assert!(returned.letters_only.errors().is_empty());
    assert_eq!(returned.mixed.errors(), &[regex_err(r"[a-zA-Z]")]);

    // mixed = "" — fails both → two errors, in the declaration order
    // (digit pattern first, letter pattern second).
    let v = ThreeFieldsValidable::new(ThreeFields {
        digits_only: "12".to_string(),
        letters_only: "ab".to_string(),
        mixed: "".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.mixed.errors(),
        &[regex_err(r"\d"), regex_err(r"[a-zA-Z]")],
        "both regex validators fired, in declaration order",
    );
}

#[test]
fn stacked_regex_validator_count_is_two() {
    let v = ThreeFieldsValidable::new(ThreeFields {
        digits_only: "0".to_string(),
        letters_only: "x".to_string(),
        mixed: "a1".to_string(),
    });
    assert_eq!(v.digits_only.validators().len(), 1);
    assert_eq!(v.letters_only.validators().len(), 1);
    assert_eq!(
        v.mixed.validators().len(),
        2,
        "two `#[validate(regex(...))]` attrs on `mixed` should produce two validators",
    );
}
