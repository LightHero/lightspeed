use std::borrow::Cow;

use lightspeed_validator::password::{PasswordError, PasswordViolation};
use lightspeed_validator::Validable;

#[derive(Validable)]
pub struct DefaultPolicy {
    #[validate(password)]
    pub password: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct RelaxedPolicy {
    #[validate(password(upper = false, number = false, special_char = false))]
    pub password: String,
}

#[derive(Validable)]
pub struct CustomSpecialChars {
    #[validate(password(upper = false, lower = false, number = false, special_char = "*$"))]
    pub password: String,
}

#[derive(Validable)]
pub struct AllowsTrailingWhitespace {
    #[validate(password(trailing_whitespaces = true))]
    pub password: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(password)]
    pub password: Cow<'static, str>,
}

fn pw_err<E: From<PasswordError>>(violations: &[PasswordViolation]) -> E {
    PasswordError { violations: violations.to_vec() }.into()
}

#[test]
fn default_policy_accepts_strong_password() {
    let v = DefaultPolicyValidable::new(DefaultPolicy {
        password: "Aa1!xyz".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn default_policy_collects_every_missing_class_in_one_error() {
    let v = DefaultPolicyValidable::new(DefaultPolicy {
        password: "aaaa".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.password.errors(),
        &[pw_err(&[
            PasswordViolation::MissingUppercase,
            PasswordViolation::MissingNumber,
            PasswordViolation::MissingSpecialChar,
        ])],
    );
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn default_policy_rejects_trailing_whitespace() {
    let v = DefaultPolicyValidable::new(DefaultPolicy {
        password: "Aa1!xyz ".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.password.errors(),
        &[pw_err(&[PasswordViolation::HasTrailingWhitespace])],
    );
}

#[test]
fn opting_out_of_classes_makes_simple_passwords_valid() {
    let v = RelaxedPolicyValidable::new(RelaxedPolicy { password: "lowercase".to_string() });
    assert!(v.validate().is_ok());
}

#[test]
fn custom_special_chars_string_restricts_accepted_set() {
    // `*` is in the custom set.
    let v = CustomSpecialCharsValidable::new(CustomSpecialChars {
        password: "abc*def".to_string(),
    });
    assert!(v.validate().is_ok());

    // `!` is in the default list but not in `"*$"`.
    let v = CustomSpecialCharsValidable::new(CustomSpecialChars {
        password: "abc!def".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(
        returned.password.errors(),
        &[pw_err(&[PasswordViolation::MissingSpecialChar])],
    );
}

#[test]
fn trailing_whitespaces_true_allows_trailing_space() {
    let v = AllowsTrailingWhitespaceValidable::new(AllowsTrailingWhitespace {
        password: "Aa1!xyz ".to_string(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn password_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields {
        password: Cow::Borrowed("Aa1!xyz"),
    });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields {
        password: Cow::Owned("aaaa".to_string()),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.password.errors().len(), 1, "one aggregated PasswordError");
}

#[test]
fn macro_attaches_one_validator_per_password_attribute() {
    let v = DefaultPolicyValidable::new(DefaultPolicy {
        password: "Aa1!xyz".to_string(),
        untouched: String::new(),
    });
    assert_eq!(v.password.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
