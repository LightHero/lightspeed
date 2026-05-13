use lightspeed_validator::{ValidableType, ValidationError, validable};

#[validable]
pub struct User {
    pub name: String,
    pub age: u32,
    pub active: bool,
}

#[test]
fn generated_struct_has_validable_typed_fields() {
    fn assert_types(v: &UserValidable) {
        let _: &ValidableType<String> = &v.name;
        let _: &ValidableType<u32> = &v.age;
        let _: &ValidableType<bool> = &v.active;
    }

    let _ = assert_types;
}

#[test]
fn validate_returns_ok_when_all_fields_are_valid() {
    let validable = UserValidable {
        name: ValidableType::new("alice".to_string()),
        age: ValidableType::new(30),
        active: ValidableType::new(true),
    };

    let user = match validable.validate() {
        Ok(user) => user,
        Err(_) => panic!("expected Ok"),
    };
    assert_eq!(user.name, "alice");
    assert_eq!(user.age, 30);
    assert!(user.active);
}

#[test]
fn validate_returns_self_when_any_field_is_invalid() {
    let mut age = ValidableType::new(30u32);
    age.push_error(ValidationError::MustBeGreater { min: "100".to_string() });

    let validable = UserValidable {
        name: ValidableType::new("alice".to_string()),
        age,
        active: ValidableType::new(true),
    };

    let returned = match validable.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(!returned.age.is_valid());
    assert!(returned.name.is_valid());
    assert!(returned.active.is_valid());
}

#[validable]
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
