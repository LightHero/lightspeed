use lightspeed_validator::range::RangeError;
use lightspeed_validator::{FieldValidator, NoError, Validable, ValidableType, ValidationError};

pub struct MinAgeContext {
    pub min_age: u32,
}

/// A custom field validator that reads the validation context. Generic over
/// `E: From<RangeError>` so it works equally well with `ValidationError`
/// (when constructed manually) and any macro-generated per-field enum that
/// includes a `Range` variant.
struct MinAgeValidator;

impl<E: From<RangeError>> FieldValidator<u32, E, MinAgeContext> for MinAgeValidator {
    fn validate(&self, value: &u32, context: &MinAgeContext) -> Result<(), E> {
        if *value >= context.min_age {
            Ok(())
        } else {
            Err(RangeError {
                min: Some(context.min_age.to_string()),
                max: None,
                exclusive_min: None,
                exclusive_max: None,
            }
            .into())
        }
    }
}

// ---- Macro-side tests ----------------------------------------------------
//
// These only verify the macro's plumbing — that `#[validate(context = ...)]`
// reaches the generated `validate(self, ctx: &<Ctx>)` signature and that the
// field types are parameterised by the context.

#[derive(Validable)]
#[validate(context = MinAgeContext)]
pub struct Person {
    pub age: u32,
    pub name: String,
}

#[test]
fn generated_field_type_uses_custom_context() {
    fn assert_field_types(v: &PersonValidable) {
        // No `#[validate(...)]` on the field → `E = NoError`.
        let _: &ValidableType<u32, NoError, MinAgeContext> = &v.age;
        let _: &ValidableType<String, NoError, MinAgeContext> = &v.name;
    }
    let _ = assert_field_types;
}

#[test]
fn validate_takes_context_argument_when_struct_opted_in() {
    // No field-level validators → always Ok, but the `validate` call must
    // accept `&MinAgeContext` for this to type-check.
    let v = PersonValidable::new(Person { age: 21, name: "alice".to_string() });
    assert!(v.validate(&MinAgeContext { min_age: 18 }).is_ok());
}

#[derive(Validable)]
pub struct DefaultCtxStruct {
    pub flag: bool,
}

#[test]
fn default_context_generates_no_arg_validate() {
    let v = DefaultCtxStruct { flag: true };
    let validable = DefaultCtxStructValidable::new(v);
    let out = match validable.validate() {
        Ok(o) => o,
        Err(_) => panic!("no validators, must be Ok"),
    };
    assert!(out.flag);
}

// ---- Direct `ValidableType<T, E, Ctx>` tests -----------------------------
//
// Exercise the actual context-forwarding plumbing inside `ValidableType` by
// constructing one directly with a custom validator that reads the context.

fn min_age_age(value: u32) -> ValidableType<u32, ValidationError, MinAgeContext> {
    ValidableType::new(value, vec![Box::new(MinAgeValidator)])
}

#[test]
fn validable_type_forwards_context_to_validators() {
    let mut age = min_age_age(21);

    age.validate(&MinAgeContext { min_age: 18 });
    assert!(age.errors().is_empty(), "21 >= 18 should pass");

    age.set(15);
    age.validate(&MinAgeContext { min_age: 18 });
    assert_eq!(
        age.errors(),
        &[ValidationError::Range(RangeError {
            min: Some("18".to_string()),
            max: None,
            exclusive_min: None,
            exclusive_max: None,
        })],
        "15 < 18 should fail with the bound from the context",
    );
}

#[test]
fn same_value_validates_differently_under_different_contexts() {
    let mut age = min_age_age(16);

    age.validate(&MinAgeContext { min_age: 13 });
    assert!(age.errors().is_empty());

    age.validate(&MinAgeContext { min_age: 18 });
    assert!(!age.errors().is_empty());
}
