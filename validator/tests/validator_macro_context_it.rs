use lightspeed_validator::range::RangeError;
use lightspeed_validator::{FieldValidator, Validable, ValidableType, ValidationError};

pub struct MinAgeContext {
    pub min_age: u32,
}

struct MinAgeValidator;

impl FieldValidator<u32, ValidationError, MinAgeContext> for MinAgeValidator {
    fn validate(&self, value: &u32, context: &MinAgeContext) -> Result<(), ValidationError> {
        if *value >= context.min_age {
            Ok(())
        } else {
            Err(ValidationError::Range(RangeError {
                min: Some(context.min_age.to_string()),
                max: None,
                exclusive_min: None,
                exclusive_max: None,
            }))
        }
    }
}

#[derive(Validable)]
#[validate(context = MinAgeContext)]
pub struct Person {
    pub age: u32,
    pub name: String,
}

#[test]
fn generated_field_type_uses_custom_context() {
    fn assert_field_types(v: &PersonValidable) {
        // Note: macro-generated fields use `ValidableType<T, ValidationError, Ctx>`
        // (no field-level validators means the per-field enum isn't generated).
        let _: &ValidableType<u32, ValidationError, MinAgeContext> = &v.age;
        let _: &ValidableType<String, ValidationError, MinAgeContext> = &v.name;
    }
    let _ = assert_field_types;
}

#[test]
fn validate_accepts_context_and_forwards_it_to_field_validators() {
    let mut v = PersonValidable::new(Person { age: 21, name: "alice".to_string() });
    v.age = ValidableType::new(21, vec![Box::new(MinAgeValidator)]);

    let person = match v.validate(&MinAgeContext { min_age: 18 }) {
        Ok(p) => p,
        Err(_) => panic!("expected Ok — 21 >= 18"),
    };
    assert_eq!(person.age, 21);
    assert_eq!(person.name, "alice");
}

#[test]
fn validate_fails_when_validator_rejects_value_for_the_context() {
    let mut v = PersonValidable::new(Person { age: 15, name: "bob".to_string() });
    v.age = ValidableType::new(15, vec![Box::new(MinAgeValidator)]);

    let returned = match v.validate(&MinAgeContext { min_age: 18 }) {
        Ok(_) => panic!("expected Err — 15 < 18"),
        Err(v) => v,
    };
    assert_eq!(
        returned.age.errors(),
        &[ValidationError::Range(RangeError {
            min: Some("18".to_string()),
            max: None,
            exclusive_min: None,
            exclusive_max: None,
        })],
    );
    assert!(returned.name.errors().is_empty());
}

#[test]
fn same_value_validates_differently_under_different_contexts() {
    let make = || {
        let mut v = PersonValidable::new(Person { age: 16, name: "carol".to_string() });
        v.age = ValidableType::new(16, vec![Box::new(MinAgeValidator)]);
        v
    };

    assert!(make().validate(&MinAgeContext { min_age: 13 }).is_ok());
    assert!(make().validate(&MinAgeContext { min_age: 18 }).is_err());
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
