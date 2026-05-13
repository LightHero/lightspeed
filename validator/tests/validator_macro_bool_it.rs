use lightspeed_validator::{Validable, ValidableType, ValidationError};

#[derive(Validable)]
pub struct Flags {
    #[validate(isTrue)]
    pub enabled: bool,
    #[validate(isFalse)]
    pub debug: bool,
}

#[test]
fn field_level_is_true_validator_rejects_false() {
    let v = FlagsValidable {
        enabled: ValidableType::new(false),
        debug: ValidableType::new(false),
    };

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.enabled.errors(), &vec![ValidationError::MustBeTrue]);
    assert!(returned.debug.is_valid());
}

#[test]
fn field_level_is_false_validator_rejects_true() {
    let v = FlagsValidable {
        enabled: ValidableType::new(true),
        debug: ValidableType::new(true),
    };

    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.enabled.is_valid());
    assert_eq!(returned.debug.errors(), &vec![ValidationError::MustBeFalse]);
}

#[test]
fn field_level_validators_pass_when_values_match() {
    let v = FlagsValidable {
        enabled: ValidableType::new(true),
        debug: ValidableType::new(false),
    };

    let flags = match v.validate() {
        Ok(f) => f,
        Err(_) => panic!("expected Ok"),
    };
    assert!(flags.enabled);
    assert!(!flags.debug);
}
