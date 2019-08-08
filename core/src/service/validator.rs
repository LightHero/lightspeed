use crate::error::{ErrorDetails, LightSpeedError};

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

impl<V0: Validable, V1: Validable> Validable for (V0, V1) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable> Validable for (V0, V1, V2) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        self.0.validate(error_details)?;
        self.1.validate(error_details)?;
        self.2.validate(error_details)
    }
}

impl<V0: Validable, V1: Validable, V2: Validable, V3: Validable> Validable for (V0, V1, V2, V3) {
    #[inline]
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6)
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7)
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7, V8)
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
    > Validable for (V0, V1, V2, V3, V4, V5, V6, V7, V8, V9)
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

pub struct Validator {}

impl Validator {
    pub fn validate<V: Validable>(validable: V) -> Result<(), LightSpeedError> {
        let mut error_details = ErrorDetails::default();
        validable.validate(&mut error_details)?;
        if !error_details.details.is_empty() {
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
        let result = Validator::validate(|_error_details: &mut ErrorDetails| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_return_error_from_closure_if_error_details() {
        let result = Validator::validate(|error_details: &mut ErrorDetails| {
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
        let result = Validator::validate(|_error_details: &mut ErrorDetails| {
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
    pub fn validator_should_accept_pair() {
        let validable1 = Tester::default();
        let validable2 = Tester::default();

        let result = Validator::validate((&validable1, &validable2));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_accept_pair_with_closures() {
        let validable1 = Tester::default();

        let result = Validator::validate((&validable1, |_error_details: &mut ErrorDetails| Ok(())));

        assert!(result.is_ok());
    }

    #[test]
    pub fn validator_should_aggregate_errors() {
        let mut error_details = ErrorDetails::default();
        error_details.add_detail("1", "2");

        let validable1 = Tester {
            error_details: error_details.clone(),
        };

        let validable2 = Tester {
            error_details: error_details.clone(),
        };

        let validable3 = Tester {
            error_details: error_details.clone(),
        };

        error_details.add_detail("2", "2");
        let validable4 = Tester {
            error_details: error_details.clone(),
        };

        let validable5 = Tester {
            error_details: error_details.clone(),
        };

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
                assert_eq!(5, details.details["1"].len());
                assert_eq!(2, details.details["2"].len());
            }
            _ => assert!(false),
        }
    }

    #[derive(Default, Clone)]
    struct Tester {
        error_details: ErrorDetails,
    }

    impl Validable for &Tester {
        fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
            for (key, details) in &self.error_details.details {
                for detail in details {
                    error_details.add_detail(key.clone(), detail.clone())
                }
            }
            Ok(())
        }
    }
}
