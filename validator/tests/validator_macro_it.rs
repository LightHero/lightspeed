
use lightspeed_validator::{ValidableType, validable};


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

        let user = User { name: "alice".to_string(), age: 30, active: true };
        assert_eq!(user.name, "alice");
        assert_eq!(user.age, 30);
        assert!(user.active);

        // Confirm the generated type name and field types compile.
        let _ = assert_types;
    }

