extern crate self as lightspeed_validator;

pub mod boolean;
pub mod contains;
pub mod email;
pub mod error;
pub mod ip;
pub mod must_match;
pub mod order;
pub mod ownership;
pub mod urls;
mod macros;

pub use error::{ErrorDetails, RootErrorDetails, ValidableType, ValidationError, ValidatorError};
pub use macros::validable;

pub trait Validable: Send + Sync {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl<F> Validable for F
where
    F: Send + Sync + Fn(&mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self)(error_details)
    }
}

impl<V0: Validable, V1: Validable> Validable for (&V0, &V1) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable> Validable for (&V0, &V1, &V2) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable> Validable for (&V0, &V1, &V2, &V3) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable> Validable
    for (&V0, &V1, &V2, &V3, &V4)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable, V5: Validable> Validable
    for (&V0, &V1, &V2, &V3, &V4, &V5)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable, V5: Validable, V6: Validable>
    Validable for (&V0, &V1, &V2, &V3, &V4, &V5, &V6)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)?;
        self.6.validate(error_details)
    }
}

impl<
    V0: Validable,
    V1: Validable,
    V2: Validable,
    V3: Validable,
    V4: Validable,
    V5: Validable,
    V6: Validable,
    V7: Validable,
> Validable for (&V0, &V1, &V2, &V3, &V4, &V5, &V6, &V7)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)?;
        self.6.validate(error_details)?;
        self.7.validate(error_details)
    }
}

impl<
    V0: Validable,
    V1: Validable,
    V2: Validable,
    V3: Validable,
    V4: Validable,
    V5: Validable,
    V6: Validable,
    V7: Validable,
    V8: Validable,
> Validable for (&V0, &V1, &V2, &V3, &V4, &V5, &V6, &V7, &V8)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)?;
        self.6.validate(error_details)?;
        self.7.validate(error_details)?;
        self.8.validate(error_details)
    }
}

impl<
    V0: Validable,
    V1: Validable,
    V2: Validable,
    V3: Validable,
    V4: Validable,
    V5: Validable,
    V6: Validable,
    V7: Validable,
    V8: Validable,
    V9: Validable,
> Validable for (&V0, &V1, &V2, &V3, &V4, &V5, &V6, &V7, &V8, &V9)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)?;
        self.6.validate(error_details)?;
        self.7.validate(error_details)?;
        self.8.validate(error_details)?;
        self.9.validate(error_details)
    }
}

#[derive(Default)]
pub struct Validator<'a> {
    error_details: ErrorDetails<'a>,
    validables: Vec<&'a dyn Validable>,
}

impl<'a> Validator<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn validate<V: Validable>(validable: &'a V) -> Result<(), ValidatorError> {
        Validator::new().on(validable).do_validate()
    }

    pub fn on<V: Validable>(mut self, validable: &'a V) -> Self {
        self.validables.push(validable);
        self
    }

    pub fn error_details(&mut self) -> &mut ErrorDetails<'a> {
        &mut self.error_details
    }

    pub fn do_validate(mut self) -> Result<(), ValidatorError> {
        for validable in self.validables {
            validable.validate(&mut self.error_details).map_err(ValidatorError::Error)?;
        }

        if !self.error_details.details().is_empty() {
            match self.error_details {
                ErrorDetails::Root(node) => Err(ValidatorError::ValidationFailed { details: node }),
                ErrorDetails::Scoped(_) => {
                    panic!("ErrorDetails must be of type Root inside validator")
                }
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn validator_should_accept_closures() {
        let result = Validator::validate(&|_error_details: &mut ErrorDetails| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_return_error_from_closure_if_error_details() {
        let result = Validator::validate(&|error_details: &mut ErrorDetails| {
            error_details.add_detail("username", ValidationError::NotUnique);
            Ok(())
        });

        assert!(result.is_err());
        match result {
            Err(ValidatorError::ValidationFailed { details }) => {
                assert_eq!(ValidationError::NotUnique, details.details["username"][0])
            }
            _ => panic!(),
        }
    }

    #[test]
    pub fn validator_should_return_validable_internal_error() {
        #[derive(Debug)]
        struct TestError;
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "test error")
            }
        }
        impl std::error::Error for TestError {}

        let result =
            Validator::validate(&|_error_details: &mut ErrorDetails| Err(Box::new(TestError) as _));

        assert!(result.is_err());
        match result {
            Err(ValidatorError::Error(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    pub fn validator_should_accept_validable() {
        let validable = Tester { error_details: ErrorDetails::default() };
        let result = Validator::validate(&validable);
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_validable_with_errors() {
        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);

        let validable = Tester { error_details };

        let result = Validator::validate(&validable);
        assert!(result.is_err());
        match result {
            Err(ValidatorError::ValidationFailed { details }) => {
                assert_eq!(ValidationError::NotUnique, details.details["1"][0])
            }
            _ => panic!(),
        }
    }

    #[test]
    pub fn validator_should_accept_validables() {
        let mut validable_1 = Tester { error_details: ErrorDetails::default() };
        validable_1.error_details.add_detail("1", ValidationError::NotUnique);

        let mut validable_2 = Tester { error_details: ErrorDetails::default() };
        validable_2.error_details.add_detail("1", ValidationError::ValueRequired);
        validable_2.error_details.add_detail("2", ValidationError::UnknownField);

        let validable_3 = |error_details: &mut ErrorDetails| {
            error_details.add_detail("3", ValidationError::MustBeTrue);
            Ok(())
        };

        let result = Validator::new().on(&validable_1).on(&validable_2).on(&validable_3).do_validate();

        assert!(result.is_err());
        match result {
            Err(ValidatorError::ValidationFailed { details }) => {
                assert_eq!(ValidationError::NotUnique, details.details["1"][0]);
                assert_eq!(ValidationError::ValueRequired, details.details["1"][1]);
                assert_eq!(ValidationError::UnknownField, details.details["2"][0]);
                assert_eq!(ValidationError::MustBeTrue, details.details["3"][0]);
            }
            _ => panic!(),
        }
    }

    #[test]
    pub fn validator_should_accept_pair() {
        let validable1 = Tester::new();
        let validable2 = Tester::new();

        let result = Validator::validate(&(&validable1, &validable2));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_pair_with_closures() {
        let validable1 = Tester::new();

        let result = Validator::validate(&(&validable1, &|_error_details: &mut ErrorDetails| Ok(())));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_aggregate_errors() {
        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);
        let validable1 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);
        let validable2 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);
        let validable3 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);
        error_details.add_detail("2", ValidationError::NotUnique);
        let validable4 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", ValidationError::NotUnique);
        error_details.add_detail("2", ValidationError::NotUnique);
        let validable5 = Tester { error_details };

        let result = Validator::validate(&(&validable1, &validable2, &validable3, &validable4, &validable5));

        assert!(result.is_err());
        match result {
            Err(ValidatorError::ValidationFailed { details }) => {
                assert_eq!(5, details.details["1"].len());
                assert_eq!(2, details.details["2"].len());
            }
            _ => panic!(),
        }
    }

    struct Tester<'a> {
        error_details: ErrorDetails<'a>,
    }

    impl Tester<'_> {
        fn new() -> Self {
            Self { error_details: ErrorDetails::default() }
        }
    }

    impl Validable for Tester<'_> {
        fn validate(
            &self,
            error_details: &mut ErrorDetails,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            for (key, details) in self.error_details.details().iter() {
                for detail in details {
                    error_details.add_detail(key.clone(), detail.clone())
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod some {

    #[test]
    fn some() {

        use serde::Deserialize;
        // A trait that the Validate derive will impl
        use validator::{Validate, ValidationError};
        
        #[derive(Validate, Deserialize)]
        pub struct Val {
            #[validate(email)]
            pub email: String,
            #[validate(url)]
            pub url: String,
        }

        let val = Val {
            email: "invalid".to_string(),
            url: "invalid".to_string(),
        };

        let err = val.validate().unwrap_err();

        println!("{:?}", err);

    }
}

#[cfg(test)]
mod validable_macro {
    use crate::validable;

    #[validable]
    pub struct User {
        pub name: String,
        pub age: u32,
        pub active: bool,
    }

    #[test]
    fn generated_struct_has_validable_typed_fields() {
        fn assert_types(v: &UserValidable) {
            let _: &crate::ValidableType<String> = &v.name;
            let _: &crate::ValidableType<u32> = &v.age;
            let _: &crate::ValidableType<bool> = &v.active;
        }

        let user = User { name: "alice".to_string(), age: 30, active: true };
        assert_eq!(user.name, "alice");
        assert_eq!(user.age, 30);
        assert!(user.active);

        // Confirm the generated type name and field types compile.
        let _ = assert_types;
    }
}
