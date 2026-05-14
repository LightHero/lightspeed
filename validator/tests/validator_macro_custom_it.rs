use std::collections::HashMap;

use lightspeed_validator::{FieldValidator, Validable, ValidationError, length::LengthError};

fn not_empty(value: &String, _ctx: &()) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(ValidationError::Custom {
            code: "not_empty".to_string(),
            message: "must not be empty".to_string(),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

#[derive(Validable)]
pub struct User {
    #[validate(custom(function = "not_empty"))]
    pub name: String,
    pub untouched: String,
}

#[test]
fn custom_validator_rejects_when_function_errs() {
    let v = UserValidable::new(User { name: String::new(), untouched: String::new() });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.name.errors().len(), 1);
    match &returned.name.errors()[0] {
        ValidationError::Custom { code, .. } => assert_eq!(code, "not_empty"),
        other => panic!("expected Custom, got {other:?}"),
    }
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn custom_validator_accepts_when_function_ok() {
    let v = UserValidable::new(User { name: "alice".to_string(), untouched: String::new() });

    let user = match v.validate() {
        Ok(u) => u,
        Err(_) => panic!("expected Ok"),
    };
    assert_eq!(user.name, "alice");
}

#[test]
fn custom_validator_emitted_as_single_validator() {
    let v = UserValidable::new(User { name: "alice".to_string(), untouched: String::new() });

    assert_eq!(v.name.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}

#[test]
fn custom_validator_invokes_the_user_function() {
    // Drive the user function through the trait object stored in the
    // generated validator vec — sanity-checks the macro emitted a runtime
    // wrapper that actually calls our function (not some default).
    let v = UserValidable::new(User { name: "alice".to_string(), untouched: String::new() });
    let validator = &v.name.validators()[0];

    assert!(validator.validate(&"alice".to_string(), &()).is_ok());
    assert!(matches!(
        validator.validate(&String::new(), &()),
        Err(ValidationError::Custom { .. }),
    ));
}

// ---- custom + context ----------------------------------------------------

pub struct MinLenContext {
    pub min: usize,
}

fn at_least_min(value: &String, ctx: &MinLenContext) -> Result<(), ValidationError> {
    if value.chars().count() >= ctx.min {
        Ok(())
    } else {
        Err(ValidationError::Length(LengthError {
            min: Some(ctx.min),
            max: None,
            equal: None,
            actual: value.chars().count(),
        }))
    }
}

#[derive(Validable)]
#[validate(context = MinLenContext)]
pub struct PolicyUser {
    #[validate(custom(function = "at_least_min"))]
    pub name: String,
}

#[test]
fn custom_validator_reads_the_struct_context() {
    let v = PolicyUserValidable::new(PolicyUser { name: "ab".to_string() });
    assert!(v.validate(&MinLenContext { min: 3 }).is_err());

    let v = PolicyUserValidable::new(PolicyUser { name: "alice".to_string() });
    assert!(v.validate(&MinLenContext { min: 3 }).is_ok());
}

// ---- custom alongside other validators ----------------------------------

fn must_be_even(value: &u32, _ctx: &()) -> Result<(), ValidationError> {
    if value % 2 == 0 {
        Ok(())
    } else {
        Err(ValidationError::Custom {
            code: "not_even".to_string(),
            message: "must be even".to_string(),
            params: HashMap::new(),
        })
    }
}

#[derive(Validable)]
pub struct Counter {
    // `range` is declared first so its error appears before the custom
    // one when both fail — this lets us assert validators run in
    // attribute order.
    #[validate(range(min = 0, max = 100))]
    #[validate(custom(function = "must_be_even"))]
    pub value: u32,
}

#[test]
fn custom_validator_composes_with_built_in_validators() {
    // Both validators fail: range (101 > 100) and custom (101 is odd).
    let v = CounterValidable::new(Counter { value: 101 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.value.errors().len(), 2);
    assert!(matches!(&returned.value.errors()[0], ValidationError::Range(_)));
    assert!(matches!(&returned.value.errors()[1], ValidationError::Custom { .. }));

    // Only the custom validator fails.
    let v = CounterValidable::new(Counter { value: 5 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.value.errors().len(), 1);
    assert!(matches!(&returned.value.errors()[0], ValidationError::Custom { .. }));

    // Both pass.
    let v = CounterValidable::new(Counter { value: 4 });
    assert!(v.validate().is_ok());
}

// ---- custom via bare path (no string literal) ---------------------------

fn not_blank(value: &String, _ctx: &()) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::Custom {
            code: "blank".to_string(),
            message: "must not be blank".to_string(),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

#[derive(Validable)]
pub struct BarePathUser {
    #[validate(custom(function = not_blank))]
    pub name: String,
}

#[test]
fn custom_validator_accepts_bare_path() {
    assert!(BarePathUserValidable::new(BarePathUser { name: "alice".to_string() }).validate().is_ok());
    assert!(BarePathUserValidable::new(BarePathUser { name: "   ".to_string() }).validate().is_err());
}

// ---- custom under `errors(tailored)` ------------------------------------
//
// In tailored mode the per-field enum gains `Custom(ValidationError)` plus
// `From<ValidationError>` — the user function returns the wide
// `ValidationError` and `.into()` lifts it into the narrow enum.

fn ban_root(value: &String, _ctx: &()) -> Result<(), TailoredUserNameFieldError> {
    if value == "root" {
        Err(ValidationError::Custom {
            code: "banned".to_string(),
            message: "name is reserved".to_string(),
            params: HashMap::new(),
        }
        .into())
    } else {
        Ok(())
    }
}

#[derive(Validable)]
#[validate(errors(tailored))]
pub struct TailoredUser {
    #[validate(custom(function = "ban_root"))]
    pub name: String,
}

#[test]
fn custom_validator_works_under_tailored_errors() {
    let v = TailoredUserValidable::new(TailoredUser { name: "root".to_string() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.name.errors().len(), 1);
    let TailoredUserNameFieldError::Custom(ValidationError::Custom { code, .. }) = &returned.name.errors()[0] else {
        panic!("expected TailoredUserNameFieldError::Custom(ValidationError::Custom)");
    };
    assert_eq!(code, "banned");

    let v = TailoredUserValidable::new(TailoredUser { name: "alice".to_string() });
    assert!(v.validate().is_ok());
}
