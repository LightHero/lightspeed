use crate::error::{ErrorDetails, LightSpeedError};

pub mod boolean;
pub mod contains;
pub mod email;
pub mod ip;
pub mod must_match;
pub mod number;
pub mod urls;

pub const ERR_NOT_UNIQUE: &str = "NOT_UNIQUE";
pub const ERR_VALUE_REQUIRED: &str = "VALUE_REQUIRED";
pub const ERR_UNKNOWN_FIELD: &str = "UNKNOWN_FIELD";

pub trait Validable {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError>;
}

impl<F> Validable for F
where
    F: Fn(&mut ErrorDetails) -> Result<(), LightSpeedError>,
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        (self)(error_details)
    }
}

impl<V0: Validable, V1: Validable> Validable for (&V0, &V1) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable> Validable for (&V0, &V1, &V2) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable> Validable
    for (&V0, &V1, &V2, &V3)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable, V5: Validable>
    Validable for (&V0, &V1, &V2, &V3, &V4, &V5)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)?;
        self.5.validate(error_details)
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
    > Validable for (&V0, &V1, &V2, &V3, &V4, &V5, &V6)
{
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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

    pub fn validate<V: Validable>(validable: &'a V) -> Result<(), LightSpeedError> {
        Validator::new().on(validable).do_validate()
    }

    pub fn on<V: Validable>(mut self, validable: &'a V) -> Self {
        self.validables.push(validable);
        self
    }

    pub fn error_details(&mut self) -> &mut ErrorDetails<'a> {
        &mut self.error_details
    }

    pub fn do_validate(mut self) -> Result<(), LightSpeedError> {
        for validable in self.validables {
            validable.validate(&mut self.error_details)?;
        }

        if !self.error_details.details().is_empty() {
            match self.error_details {
                ErrorDetails::Root(node) => Err(LightSpeedError::ValidationError { details: node }),
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
            error_details.add_detail("username", "duplicated");
            Ok(())
        });

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("duplicated", details.details["username"][0])
            }
            _ => assert!(false),
        }
    }

    #[test]
    pub fn validator_should_return_validable_internal_error() {
        let result = Validator::validate(&|_error_details: &mut ErrorDetails| {
            Err(LightSpeedError::UnauthenticatedError)
        });

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::UnauthenticatedError) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    pub fn validator_should_accept_validable() {
        let validable = Tester {
            error_details: ErrorDetails::default(),
        };
        let result = Validator::validate(&validable);
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_validable_with_errors() {
        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");

        let validable = Tester { error_details };

        let result = Validator::validate(&validable);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("2", details.details["1"][0])
            }
            _ => assert!(false),
        }
    }

    #[test]
    pub fn validator_should_accept_validables() {
        let mut validable_1 = Tester {
            error_details: ErrorDetails::default(),
        };
        validable_1.error_details.add_detail("1", "11");

        let mut validable_2 = Tester {
            error_details: ErrorDetails::default(),
        };
        validable_2.error_details.add_detail("1", "111");
        validable_2.error_details.add_detail("2", "22");

        let validable_3 = |error_details: &mut ErrorDetails| {
            error_details.add_detail("3", "33");
            Ok(())
        };

        let result = Validator::new()
            .on(&validable_1)
            .on(&validable_2)
            .on(&validable_3)
            .do_validate();

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("11", details.details["1"][0]);
                assert_eq!("111", details.details["1"][1]);
                assert_eq!("22", details.details["2"][0]);
                assert_eq!("33", details.details["3"][0]);
            }
            _ => assert!(false),
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

        let result =
            Validator::validate(&(&validable1, &|_error_details: &mut ErrorDetails| Ok(())));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_aggregate_errors() {
        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");
        let validable1 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");
        let validable2 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");
        let validable3 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");
        error_details.add_detail("2", "2");
        let validable4 = Tester { error_details };

        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");
        error_details.add_detail("2", "2");
        let validable5 = Tester { error_details };

        let result = Validator::validate(&(
            &validable1,
            &validable2,
            &validable3,
            &validable4,
            &validable5,
        ));

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(5, details.details["1"].len());
                assert_eq!(2, details.details["2"].len());
            }
            _ => assert!(false),
        }
    }

    struct Tester<'a> {
        error_details: ErrorDetails<'a>,
    }

    impl<'a> Tester<'a> {
        fn new() -> Self {
            Self {
                error_details: ErrorDetails::default(),
            }
        }
    }

    impl<'a> Validable for Tester<'a> {
        fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
            for (key, details) in self.error_details.details().iter() {
                for detail in details {
                    error_details.add_detail(key.clone(), detail.clone())
                }
            }
            Ok(())
        }
    }
}
