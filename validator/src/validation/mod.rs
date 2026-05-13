use crate::ValidationError;

pub mod boolean;
pub mod contains;
#[cfg(feature = "credit_card")]
pub mod credit_card;
pub mod fields_match;
pub mod ip;
pub mod url;

pub trait FieldValidator<T, E, CTX> {
    fn validate(&self, value: &T, context: &CTX) -> Result<(), E>;
}

impl<T, E, CTX> FieldValidator<T, E, CTX> for fn(&T, &CTX) -> Result<(), E> {
    fn validate(&self, value: &T, context: &CTX) -> Result<(), E> {
        self(value, context)
    }
}

/// Runs after every field-level validator. Receives a reference to the whole
/// validable struct so it can inspect multiple fields at once. Returns the
/// collected errors as a `Vec<E>` (empty on success).
pub trait StructValidator<T, E, CTX> {
    fn validate(&self, value: &T, context: &CTX) -> Result<(), Vec<E>>;
}

pub struct ValidableType<T, Ctx = ()> {
    value: T,
    validators: Vec<Box<dyn FieldValidator<T, ValidationError, Ctx>>>,
    errors: Vec<ValidationError>,
}

impl<T, Ctx> ValidableType<T, Ctx> {
    
    pub fn new(value: T, validators: Vec<Box<dyn FieldValidator<T, ValidationError, Ctx>>>) -> Self {
        Self { value, validators, errors: vec![] }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn validators(&self) -> &[Box<dyn FieldValidator<T, ValidationError, Ctx>>] {
        &self.validators
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn push_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn validate(&mut self, ctx: &Ctx) {
        self.errors.clear();
        for validator in &self.validators {
            if let Err(e) = validator.validate(&self.value, ctx) {
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
    fn new_does_not_auto_validate() {
        let validable = ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 })]);
        assert!(validable.errors().is_empty());
        assert_eq!(&3, validable.get());
    }

    #[test]
    fn validate_with_no_validators_leaves_errors_empty() {
        let mut validable: ValidableType<usize> = ValidableType::new(10, vec![]);
        validable.validate(&());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn validate_with_passing_validator_leaves_errors_empty() {
        let mut validable = ValidableType::new(
            10,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        validable.validate(&());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn validate_with_failing_validator_collects_error() {
        let mut validable = ValidableType::new(
            3,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        validable.validate(&());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 5 }, &validable.errors()[0]);
    }

    #[test]
    fn validate_collects_errors_from_all_failing_validators() {
        let mut validable = ValidableType::new(
            3,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }),
                Box::new(MustBeGreaterValidator { min: 10 }),
            ],
        );
        validable.validate(&());
        assert_eq!(2, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 5 }, &validable.errors()[0]);
        assert_eq!(&ValidationError::MustBeGreater { min: 10 }, &validable.errors()[1]);
    }

    #[test]
    fn validate_mixed_passing_and_failing_validators() {
        let mut validable = ValidableType::new(
            7,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }),
                Box::new(MustBeGreaterValidator { min: 10 }),
            ],
        );
        validable.validate(&());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&ValidationError::MustBeGreater { min: 10 }, &validable.errors()[0]);
    }

    #[test]
    fn validate_clears_previous_errors_on_rerun() {
        let mut validable = ValidableType::new(
            3,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        validable.validate(&());
        assert_eq!(1, validable.errors().len());

        validable.set(10);
        validable.validate(&());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn set_updates_value_without_revalidating() {
        let mut validable = ValidableType::new(
            3,
            vec![Box::new(MustBeGreaterValidator { min: 5 })],
        );
        validable.set(10);
        assert_eq!(&10, validable.get());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn into_value_returns_owned_value() {
        let validable: ValidableType<String> = ValidableType::new("hello".to_string(), vec![]);
        let value = validable.into_value();
        assert_eq!("hello".to_string(), value);
    }

    struct MinValidator {
        floor: usize,
    }

    impl FieldValidator<usize, ValidationError, usize> for MinValidator {
        fn validate(&self, value: &usize, context: &usize) -> Result<(), ValidationError> {
            if *value >= self.floor + *context {
                Ok(())
            } else {
                Err(ValidationError::MustBeGreater { min: self.floor + *context })
            }
        }
    }

    #[test]
    fn validate_forwards_context_to_validators() {
        let mut validable: ValidableType<usize, usize> =
            ValidableType::new(8, vec![Box::new(MinValidator { floor: 5 })]);

        validable.validate(&2);
        assert!(validable.errors().is_empty(), "8 >= 5 + 2 should pass");

        validable.validate(&5);
        assert_eq!(
            validable.errors(),
            &[ValidationError::MustBeGreater { min: 10 }],
            "8 < 5 + 5 should fail",
        );
    }
}
