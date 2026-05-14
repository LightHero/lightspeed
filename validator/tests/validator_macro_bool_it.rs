use lightspeed_validator::Validable;
use lightspeed_validator::boolean::{MustBeFalseError, MustBeTrueError};

// These tests reference the per-field enum names (e.g. `FlagsEnabledFieldError`)
// directly, so the struct opts into the `tailored` error strategy.
#[derive(Validable)]
#[validate(errors(tailored))]
pub struct Flags {
    #[validate(isTrue)]
    pub enabled: bool,
    #[validate(isFalse)]
    pub debug: bool,
    pub untouched: bool,
}

#[derive(Validable)]
#[validate(errors(tailored))]
pub struct MultiAttrFlags {
    #[validate(isTrue)]
    #[validate(isFalse)]
    pub via_multiple_attrs: bool,
    #[validate(isTrue, isFalse)]
    pub via_single_attr: bool,
}

#[test]
fn field_level_is_true_validator_rejects_false() {
    let v = FlagsValidable::new(Flags { enabled: false, debug: false, untouched: false });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.enabled.errors(), &[FlagsEnabledFieldError::MustBeTrue(MustBeTrueError)],);
    assert!(returned.debug.errors().is_empty());
}

#[test]
fn field_level_is_false_validator_rejects_true() {
    let v = FlagsValidable::new(Flags { enabled: true, debug: true, untouched: false });

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.enabled.errors().is_empty());
    assert_eq!(returned.debug.errors(), &[FlagsDebugFieldError::MustBeFalse(MustBeFalseError)],);
}

#[test]
fn field_level_validators_pass_when_values_match() {
    let v = FlagsValidable::new(Flags { enabled: true, debug: false, untouched: false });

    let flags = match v.validate() {
        Ok(f) => f,
        Err(_) => panic!("expected Ok"),
    };
    assert!(flags.enabled);
    assert!(!flags.debug);
}

#[test]
fn macro_attaches_one_validator_per_validate_keyword() {
    let v = FlagsValidable::new(Flags { enabled: true, debug: false, untouched: false });

    assert_eq!(v.enabled.validators().len(), 1);
    assert_eq!(v.debug.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}

#[test]
fn is_true_validator_emitted_for_is_true_attribute() {
    let v = FlagsValidable::new(Flags { enabled: true, debug: false, untouched: false });
    let validator = &v.enabled.validators()[0];

    assert_eq!(validator.validate(&true, &()), Ok(()));
    assert_eq!(validator.validate(&false, &()), Err(FlagsEnabledFieldError::MustBeTrue(MustBeTrueError)),);
}

#[test]
fn is_false_validator_emitted_for_is_false_attribute() {
    let v = FlagsValidable::new(Flags { enabled: true, debug: false, untouched: false });
    let validator = &v.debug.validators()[0];

    assert_eq!(validator.validate(&false, &()), Ok(()));
    assert_eq!(validator.validate(&true, &()), Err(FlagsDebugFieldError::MustBeFalse(MustBeFalseError)),);
}

#[test]
fn macro_accepts_multiple_validate_attributes_on_same_field() {
    let v = MultiAttrFlags { via_multiple_attrs: true, via_single_attr: true };
    let validable = MultiAttrFlagsValidable::new(v);

    assert_eq!(validable.via_multiple_attrs.validators().len(), 2);
    assert_eq!(validable.via_single_attr.validators().len(), 2);
}

#[test]
fn multiple_validators_emit_each_failure() {
    let validable = MultiAttrFlagsValidable::new(MultiAttrFlags { via_multiple_attrs: true, via_single_attr: false });

    let validable = match validable.validate() {
        Ok(_) => panic!("expected at least one failure"),
        Err(v) => v,
    };

    assert_eq!(
        validable.via_multiple_attrs.errors(),
        &[MultiAttrFlagsViaMultipleAttrsFieldError::MustBeFalse(MustBeFalseError)],
        "isTrue passes, isFalse fails for value = true",
    );
    assert_eq!(
        validable.via_single_attr.errors(),
        &[MultiAttrFlagsViaSingleAttrFieldError::MustBeTrue(MustBeTrueError)],
        "isTrue fails, isFalse passes for value = false",
    );
}

#[test]
fn validators_run_in_attribute_order() {
    let v = FlagsValidable::new(Flags { enabled: false, debug: false, untouched: false });
    let mut validator_errors = Vec::new();
    for validator in v.enabled.validators() {
        if let Err(e) = validator.validate(v.enabled.get(), &()) {
            validator_errors.push(e);
        }
    }

    assert_eq!(validator_errors, vec![FlagsEnabledFieldError::MustBeTrue(MustBeTrueError)],);
}
