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
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError>;
}

impl<F> Validable for F
where
    F: Fn(&ErrorDetails) -> Result<(), LightSpeedError>,
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        (self)(error_details)
    }
}

impl<V0: Validable, V1: Validable> Validable for (V0, V1) {
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable> Validable for (V0, V1, V2) {
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable> Validable for (V0, V1, V2, V3) {
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable> Validable
    for (V0, V1, V2, V3, V4)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)?;
        self.3.validate(error_details)?;
        self.4.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable, V4: Validable, V5: Validable>
    Validable for (V0, V1, V2, V3, V4, V5)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7, V8)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7, V8, V9)
{
    #[inline]
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
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

pub struct Validator {}

impl Validator {
    pub fn validate<V: Validable>(validable: V) -> Result<(), LightSpeedError> {
        let error_details = ErrorDetails::new();
        validable.validate(&error_details)?;

        if !error_details.details().borrow().is_empty() {
            Err(LightSpeedError::ValidationError {
                details: error_details,
            })
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
        let result = Validator::validate(|_error_details: &ErrorDetails| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_return_error_from_closure_if_error_details() {
        let result = Validator::validate(|error_details: &ErrorDetails| {
            error_details.add_detail("username", "duplicated");
            Ok(())
        });

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("duplicated", details.details().borrow()["username"][0])
            }
            _ => assert!(false),
        }
    }

    #[test]
    pub fn validator_should_return_validable_internal_error() {
        let result = Validator::validate(|_error_details: &ErrorDetails| {
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
            error_details: ErrorDetails::new(),
        };
        let result = Validator::validate(&validable);
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_validable_with_errors() {
        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");

        let validable = Tester { error_details };

        let result = Validator::validate(&validable);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!("2", details.details().borrow()["1"][0])
            }
            _ => assert!(false),
        }
    }

    #[test]
    pub fn validator_should_accept_pair() {
        let validable1 = Tester::new();
        let validable2 = Tester::new();

        let result = Validator::validate((&validable1, &validable2));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_pair_with_closures() {
        let validable1 = Tester::new();

        let result = Validator::validate((&validable1, |_error_details: &ErrorDetails| Ok(())));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_aggregate_errors() {
        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");
        let validable1 = Tester { error_details };

        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");
        let validable2 = Tester { error_details };

        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");
        let validable3 = Tester { error_details };

        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");
        error_details.add_detail("2", "2");
        let validable4 = Tester { error_details };

        let error_details = ErrorDetails::new();
        error_details.add_detail("1", "2");
        error_details.add_detail("2", "2");
        let validable5 = Tester { error_details };

        let result = Validator::validate((
            &validable1,
            &validable2,
            &validable3,
            &validable4,
            &validable5,
        ));

        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(5, details.details().borrow()["1"].len());
                assert_eq!(2, details.details().borrow()["2"].len());
            }
            _ => assert!(false),
        }
    }

    struct Tester {
        error_details: ErrorDetails,
    }

    impl Tester {
        fn new() -> Self {
            Self {
                error_details: ErrorDetails::new(),
            }
        }
    }

    impl Validable for &Tester {
        fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
            let map = self.error_details.details().borrow();
            for (key, details) in map.iter() {
                for detail in details {
                    error_details.add_detail(key.clone(), detail.clone())
                }
            }
            Ok(())
        }
    }
}
