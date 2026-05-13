use lightspeed_validator::{Validable, ValidableType, boolean::MustBeFalseValidator};

#[derive(Validable)]
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
fn new_wraps_fields_in_validable_type() {
    let v = UserValidable::new(User {
        name: "alice".to_string(),
        age: 30,
        active: true,
    });

    assert_eq!(v.name.get(), "alice");
    assert_eq!(v.age.get(), &30);
    assert_eq!(v.active.get(), &true);
}

#[test]
fn validate_returns_ok_when_all_fields_are_valid() {
    let validable = UserValidable::new(User {
        name: "alice".to_string(),
        age: 30,
        active: true,
    });

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
    let v = UserValidable::new(User {
        name: "alice".to_string(),
        age: 30,
        active: true,
    });

    assert!(v.name.validators().is_empty());
    assert!(v.age.validators().is_empty());
    assert!(v.active.validators().is_empty());
}

#[test]
fn fields_without_validators_are_always_valid() {
    let v = UserValidable::new(User {
        name: String::new(),
        age: 0,
        active: false,
    });

    assert!(v.name.errors().is_empty());
    assert!(v.age.errors().is_empty());
    assert!(v.active.errors().is_empty());

    assert!(v.validate().is_ok());
}

#[test]
fn test_if_a_field_has_an_error_validatios_fails() {
    {
        let mut v = UserValidable::new(User {
            name: "alice".to_string(),
            age: 30,
            active: true,
        });

        v.active = ValidableType::new(false, vec![Box::new(MustBeFalseValidator {})]);
        assert!(v.validate().is_ok());
    }

        {
        let mut v = UserValidable::new(User {
            name: "alice".to_string(),
            age: 30,
            active: true,
        });

        v.active = ValidableType::new(false, vec![Box::new(MustBeFalseValidator {})]);
        v.active.set(true);
        assert!(v.validate().is_err());
    }

    {
        let mut v = UserValidable::new(User {
            name: "alice".to_string(),
            age: 30,
            active: true,
        });

        v.active = ValidableType::new(true, vec![Box::new(MustBeFalseValidator {})]);
        assert!(v.validate().is_err());
    }


}

