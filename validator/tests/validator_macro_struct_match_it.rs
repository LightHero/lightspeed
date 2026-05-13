use lightspeed_validator::contains::MustContainError;
use lightspeed_validator::fields_match::{FieldsMustMatch, MustMatchField};
use lightspeed_validator::{Validable, ValidationError};

fn must_contain_err<E: From<MustContainError>>(pattern: &str, case_sensitive: bool) -> E {
    MustContainError { pattern: pattern.to_string(), case_sensitive }.into()
}

#[derive(Validable)]
#[validate(fields_match(password, password_confirm))]
pub struct Signup {
    pub password: String,
    pub password_confirm: String,
}

#[derive(Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
pub struct SignupAttach {
    pub password: String,
    pub password_confirm: String,
}

#[derive(Validable)]
#[validate(fields_match(a, b))]
#[validate(fields_match(c, d, attach_to_fields = true))]
pub struct MultiRule {
    pub a: String,
    pub b: String,
    pub c: String,
    pub d: String,
}

#[derive(Validable)]
#[validate(fields_match(start, end))]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

#[derive(Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
pub struct WithFieldRules {
    #[validate(contains(pattern = "@"))]
    pub password: String,
    pub password_confirm: String,
}

fn fields_must_match<E: From<FieldsMustMatch>>(a: &str, b: &str) -> E {
    FieldsMustMatch { field_a: a.to_string(), field_b: b.to_string() }.into()
}

fn must_match_field<E: From<MustMatchField>>(other: &str) -> E {
    MustMatchField { field: other.to_string() }.into()
}

#[test]
fn fields_match_passes_when_equal() {
    let v = SignupValidable::new(Signup {
        password: "secret".to_string(),
        password_confirm: "secret".to_string(),
    });
    let ok = match v.validate() {
        Ok(s) => s,
        Err(_) => panic!("expected Ok"),
    };
    assert_eq!(ok.password, "secret");
}

#[test]
fn fields_match_default_pushes_fields_must_match_to_top_level() {
    let v = SignupValidable::new(Signup {
        password: "a".to_string(),
        password_confirm: "b".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert_eq!(returned.top_level_errors(), &[fields_must_match("password", "password_confirm")]);
    assert!(returned.password.errors().is_empty());
    assert!(returned.password_confirm.errors().is_empty());
}

#[test]
fn fields_match_attach_to_fields_pushes_must_match_field_to_each_field() {
    let v = SignupAttachValidable::new(SignupAttach {
        password: "a".to_string(),
        password_confirm: "b".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert!(returned.top_level_errors().is_empty());
    assert_eq!(
        returned.password.errors(),
        &[must_match_field("password_confirm")],
        "password's error should point at the other field",
    );
    assert_eq!(
        returned.password_confirm.errors(),
        &[must_match_field("password")],
        "password_confirm's error should point at the other field",
    );
}

#[test]
fn fresh_validable_has_empty_top_level_errors() {
    let v = SignupValidable::new(Signup {
        password: "hi".to_string(),
        password_confirm: "hi".to_string(),
    });
    assert!(v.top_level_errors().is_empty());
}

#[test]
fn multiple_struct_rules_are_each_evaluated_independently() {
    let v = MultiRuleValidable::new(MultiRule {
        a: "x".to_string(),
        b: "y".to_string(),
        c: "p".to_string(),
        d: "q".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert_eq!(
        returned.top_level_errors(),
        &[fields_must_match("a", "b")],
        "rule on (a, b) routes to top-level",
    );
    assert_eq!(returned.c.errors(), &[must_match_field("d")]);
    assert_eq!(returned.d.errors(), &[must_match_field("c")]);
    assert!(returned.a.errors().is_empty());
    assert!(returned.b.errors().is_empty());
}

#[test]
fn multiple_struct_rules_pass_together() {
    let v = MultiRuleValidable::new(MultiRule {
        a: "x".to_string(),
        b: "x".to_string(),
        c: "y".to_string(),
        d: "y".to_string(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn fields_match_works_on_non_string_types() {
    let ok = RangeValidable::new(Range { start: 5, end: 5 });
    assert!(ok.validate().is_ok());

    let bad = RangeValidable::new(Range { start: 1, end: 9 });
    let returned = match bad.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.top_level_errors(), &[fields_must_match("start", "end")]);
}

#[test]
fn struct_rule_runs_after_field_rules_and_both_can_fail() {
    let v = WithFieldRulesValidable::new(WithFieldRules {
        password: "no-at-sign".to_string(),
        password_confirm: "different".to_string(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };

    assert_eq!(
        returned.password.errors(),
        &[must_contain_err("@", true), must_match_field("password_confirm")],
        "field-level error first, then attached struct-level error pointing at the other field",
    );
    assert_eq!(returned.password_confirm.errors(), &[must_match_field("password")]);
    assert!(returned.top_level_errors().is_empty());
}

#[test]
fn validate_returns_ok_when_field_and_struct_rules_pass() {
    let v = WithFieldRulesValidable::new(WithFieldRules {
        password: "user@example.com".to_string(),
        password_confirm: "user@example.com".to_string(),
    });
    assert!(v.validate().is_ok());
}
