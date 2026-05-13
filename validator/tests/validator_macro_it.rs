use lightspeed_validator::{NoError, Validable, ValidableType};

#[derive(Validable)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub active: bool,
}

#[test]
fn generated_struct_has_validable_typed_fields() {
    fn assert_types(v: &UserValidable) {
        // No `#[validate(...)]` attributes means each field's `E` is the
        // uninhabited `NoError` — errors can't be constructed at all.
        let _: &ValidableType<String, NoError, ()> = &v.name;
        let _: &ValidableType<u32, NoError, ()> = &v.age;
        let _: &ValidableType<bool, NoError, ()> = &v.active;
    }

    let _ = assert_types;
}

#[test]
fn new_wraps_fields_in_validable_type() {
    let v = UserValidable::new(User { name: "alice".to_string(), age: 30, active: true });

    assert_eq!(v.name.get(), "alice");
    assert_eq!(v.age.get(), &30);
    assert_eq!(v.active.get(), &true);
}

#[test]
fn validate_returns_ok_when_all_fields_are_valid() {
    let validable = UserValidable::new(User { name: "alice".to_string(), age: 30, active: true });

    let user = match validable.validate() {
        Ok(user) => user,
        Err(_) => panic!("expected Ok"),
    };
    assert_eq!(user.name, "alice");
    assert_eq!(user.age, 30);
    assert!(user.active);
}

#[test]
fn fields_without_validate_attribute_have_no_validators() {
    let v = UserValidable::new(User { name: "alice".to_string(), age: 30, active: true });

    assert!(v.name.validators().is_empty());
    assert!(v.age.validators().is_empty());
    assert!(v.active.validators().is_empty());
}

#[test]
fn fields_without_validators_are_always_valid() {
    let v = UserValidable::new(User { name: String::new(), age: 0, active: false });

    assert!(v.name.errors().is_empty());
    assert!(v.age.errors().is_empty());
    assert!(v.active.errors().is_empty());

    assert!(v.validate().is_ok());
}

#[derive(Validable)]
pub struct UserMustBeInactive {
    pub name: String,
    pub age: u32,
    #[validate(isFalse)]
    pub active: bool,
}

#[test]
fn test_if_a_field_has_an_error_validatios_fails() {
    // active = false satisfies isFalse → Ok
    let v = UserMustBeInactiveValidable::new(UserMustBeInactive { name: "alice".to_string(), age: 30, active: false });
    assert!(v.validate().is_ok());

    // mutate via `set` to make it fail
    let mut v =
        UserMustBeInactiveValidable::new(UserMustBeInactive { name: "alice".to_string(), age: 30, active: false });
    v.active.set(true);
    assert!(v.validate().is_err());

    // construct already-bad → Err
    let v = UserMustBeInactiveValidable::new(UserMustBeInactive { name: "alice".to_string(), age: 30, active: true });
    assert!(v.validate().is_err());
}

#[derive(Validable)]
pub struct MatchOnValidator {
    pub zero_validators: String,
    #[validate(contains(pattern = "@"))]
    pub one_validator: String,
    #[validate(contains(pattern = "secret"))]
    #[validate(password)]
    #[validate(length(min = 3, max = 20))]
    pub three_validators: String,
}

#[test]
fn test_match_on_no_validators() {
    let v = MatchOnValidatorValidable::new(MatchOnValidator {
        zero_validators: String::new(),
        one_validator: String::new(),
        three_validators: String::new(),
    });

    // `zero_validators` has no `#[validate(...)]` attributes so the macro
    // gives it `ValidableType<String, NoError, _>`. `NoError` is empty, so
    // the match below is trivially exhaustive — the loop body can never run.
    for _err in v.zero_validators.errors() {
        // Cannot match because _err has no variant
    }

    for err in v.one_validator.errors() {
        match err {
            MatchOnValidatorOneValidatorFieldError::MustContain(_) => {}
        }
    }

    for err in v.three_validators.errors() {
        match err {
            MatchOnValidatorThreeValidatorsFieldError::MustContain(_) => {}
            MatchOnValidatorThreeValidatorsFieldError::Password(_) => {}
            MatchOnValidatorThreeValidatorsFieldError::Length(_) => {}
        }
    }
}
