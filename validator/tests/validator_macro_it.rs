use lightspeed_validator::{Validable, ValidableType, ValidationError};

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
    age.push_error(ValidationError::MustBeGreater { min: 100 });

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
