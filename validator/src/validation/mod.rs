use crate::ValidationError;

pub mod boolean;

pub trait FieldValidator<T, E, CTX> {
    fn validate(&self, value: &T, context: &CTX) -> Result<(), E>;
}

pub struct ValidableType<T> {
    value: T,
    validators: Vec<Box<dyn FieldValidator<T, ValidationError, ()>>>,
    errors: Vec<ValidationError>,
}

impl<T> ValidableType<T> {
    
    pub fn new(value: T, validators: Vec<Box<dyn FieldValidator<T, ValidationError, ()>>>) -> Self {
        let mut value = Self { value, validators, errors: vec![] };
        value.validate();
        value
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.validate();
    }

    pub fn errors(&self) -> &Vec<ValidationError> {
        &self.errors
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn validate(&mut self) {
        self.errors.clear();
        for validator in &self.validators {
            if let Err(e) = validator.validate(&self.value, &()) {
                self.errors.push(e);
            }
        }
    }


    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MustBeGreaterValidator {
        min: usize,
    }

    impl FieldValidator<usize, ValidationError, ()> for MustBeGreaterValidator {
        fn validate(&self, value: &usize, _context: &()) -> Result<(), ValidationError> {
            if *value > self.min {
                Ok(())
            } else {
                Err(ValidationError::MustBeGreater { min: self.min })
            }
        }
    }

    #[test]
    fn new_with_no_validators_is_valid() {
        let validable: ValidableType<usize> = ValidableType::new(10, vec![]);
        assert!(validable.is_valid());
        assert!(validable.errors().is_empty());
        assert_eq!(&10, validable.get());
    }

    #[test]
    fn new_with_passing_validator_is_valid() {
        let validable = ValidableType::new(
            10,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        assert!(validable.is_valid());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn new_with_failing_validator_collects_error() {
        let validable = ValidableType::new(
            3,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        assert!(!validable.is_valid());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 5 }, &validable.errors()[0]);
    }

    #[test]
    fn collects_errors_from_all_failing_validators() {
        let validable = ValidableType::new(
            3,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }),
                Box::new(MustBeGreaterValidator { min: 10 }),
            ],
        );
        assert!(!validable.is_valid());
        assert_eq!(2, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 5 }, &validable.errors()[0]);
        assert_eq!(&ValidationError::MustBeGreater { min: 10 }, &validable.errors()[1]);
    }

    #[test]
    fn mixed_passing_and_failing_validators() {
        let validable = ValidableType::new(
            7,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }),
                Box::new(MustBeGreaterValidator { min: 10 }),
            ],
        );
        assert!(!validable.is_valid());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 10 }, &validable.errors()[0]);
    }

    #[test]
    fn set_updates_value_and_revalidates_to_valid() {
        let mut validable = ValidableType::new(
            3,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        assert!(!validable.is_valid());

        validable.set(10);
        assert_eq!(&10, validable.get());
        assert!(validable.is_valid());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn set_updates_value_and_revalidates_to_invalid() {
        let mut validable = ValidableType::new(
            10,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        assert!(validable.is_valid());

        validable.set(2);
        assert_eq!(&2, validable.get());
        assert!(!validable.is_valid());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 5 }, &validable.errors()[0]);
    }

    #[test]
    fn set_clears_previous_errors() {
        let mut validable = ValidableType::new(
            1,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }),
                Box::new(MustBeGreaterValidator { min: 10 }),
            ],
        );
        assert_eq!(2, validable.errors().len());

        validable.set(100);
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn into_value_returns_owned_value() {
        let validable = ValidableType::new("hello".to_string(), vec![]);
        let value = validable.into_value();
        assert_eq!("hello".to_string(), value);
    }
}
