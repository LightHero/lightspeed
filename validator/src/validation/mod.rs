use crate::ValidationError;

pub mod boolean;
pub mod contains;
#[cfg(feature = "credit_card")]
pub mod credit_card;
pub mod email;
pub mod fields_match;
pub mod ip;
pub mod length;
pub mod password;
pub mod range;
pub mod regex;
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

/// How a [`FieldValidator`] is stored inside [`ValidableType`].
pub enum ValidatorRef<T: 'static, E: 'static, Ctx: 'static> {
    Static(&'static dyn FieldValidator<T, E, Ctx>),
    Boxed(Box<dyn FieldValidator<T, E, Ctx>>),
}

impl<T: 'static, E: 'static, Ctx: 'static> ValidatorRef<T, E, Ctx> {
    pub fn as_validator(&self) -> &dyn FieldValidator<T, E, Ctx> {
        match self {
            Self::Static(v) => *v,
            Self::Boxed(b) => b.as_ref(),
        }
    }
}

impl<T: 'static, E: 'static, Ctx: 'static> FieldValidator<T, E, Ctx> for ValidatorRef<T, E, Ctx> {
    fn validate(&self, value: &T, context: &Ctx) -> Result<(), E> {
        self.as_validator().validate(value, context)
    }
}

impl<T: 'static, E: 'static, Ctx: 'static> From<Box<dyn FieldValidator<T, E, Ctx>>> for ValidatorRef<T, E, Ctx> {
    fn from(b: Box<dyn FieldValidator<T, E, Ctx>>) -> Self {
        Self::Boxed(b)
    }
}

impl<T: 'static, E: 'static, Ctx: 'static> From<&'static dyn FieldValidator<T, E, Ctx>> for ValidatorRef<T, E, Ctx> {
    fn from(s: &'static dyn FieldValidator<T, E, Ctx>) -> Self {
        Self::Static(s)
    }
}

/// `From<Box<V>>` for any concrete validator type.
impl<V, T: 'static, E: 'static, Ctx: 'static> From<Box<V>> for ValidatorRef<T, E, Ctx>
where
    V: FieldValidator<T, E, Ctx> + 'static,
{
    fn from(b: Box<V>) -> Self {
        Self::Boxed(b as Box<dyn FieldValidator<T, E, Ctx>>)
    }
}

pub struct ValidableType<T: 'static, E: 'static = ValidationError, Ctx: 'static = ()> {
    value: T,
    validators: Vec<ValidatorRef<T, E, Ctx>>,
    errors: Vec<E>,
}

impl<T: 'static, E: 'static, Ctx: 'static> ValidableType<T, E, Ctx> {
    /// Construct a `ValidableType` with an explicit list of [`ValidatorRef`]s.
    pub fn new(value: T, validators: Vec<ValidatorRef<T, E, Ctx>>) -> Self {
        Self { value, validators, errors: vec![] }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn validators(&self) -> &[ValidatorRef<T, E, Ctx>] {
        &self.validators
    }

    pub fn errors(&self) -> &[E] {
        &self.errors
    }

    pub fn push_error(&mut self, error: E) {
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
    use crate::validation::range::RangeError;

    /// Helper that builds the `Range` error these tests expect when a
    /// `MustBeGreaterValidator { min: N }` rejects a value — its semantic is
    /// `value > N`, which is `exclusive_min`.
    fn must_be_greater_err(min: usize) -> ValidationError {
        ValidationError::Range(RangeError {
            min: None,
            max: None,
            exclusive_min: Some(min.to_string()),
            exclusive_max: None,
        })
    }

    struct MustBeGreaterValidator {
        min: usize,
    }

    impl FieldValidator<usize, ValidationError, ()> for MustBeGreaterValidator {
        fn validate(&self, value: &usize, _context: &()) -> Result<(), ValidationError> {
            if *value > self.min { Ok(()) } else { Err(must_be_greater_err(self.min)) }
        }
    }

    #[test]
    fn new_does_not_auto_validate() {
        let validable = ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
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
        let mut validable = ValidableType::new(10, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
        validable.validate(&());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn validate_with_failing_validator_collects_error() {
        let mut validable = ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
        validable.validate(&());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&must_be_greater_err(5), &validable.errors()[0]);
    }

    #[test]
    fn validate_collects_errors_from_all_failing_validators() {
        let mut validable = ValidableType::new(
            3,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }).into(),
                Box::new(MustBeGreaterValidator { min: 10 }).into(),
            ],
        );
        validable.validate(&());
        assert_eq!(2, validable.errors().len());
        assert_eq!(&must_be_greater_err(5), &validable.errors()[0]);
        assert_eq!(&must_be_greater_err(10), &validable.errors()[1]);
    }

    #[test]
    fn validate_mixed_passing_and_failing_validators() {
        let mut validable = ValidableType::new(
            7,
            vec![
                Box::new(MustBeGreaterValidator { min: 5 }).into(),
                Box::new(MustBeGreaterValidator { min: 10 }).into(),
            ],
        );
        validable.validate(&());
        assert_eq!(1, validable.errors().len());
        assert_eq!(&must_be_greater_err(10), &validable.errors()[0]);
    }

    #[test]
    fn validate_clears_previous_errors_on_rerun() {
        let mut validable = ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
        validable.validate(&());
        assert_eq!(1, validable.errors().len());

        validable.set(10);
        validable.validate(&());
        assert!(validable.errors().is_empty());
    }

    #[test]
    fn set_updates_value_without_revalidating() {
        let mut validable = ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
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
            let min = self.floor + *context;
            if *value >= min {
                Ok(())
            } else {
                Err(ValidationError::Range(RangeError {
                    min: Some(min.to_string()),
                    max: None,
                    exclusive_min: None,
                    exclusive_max: None,
                }))
            }
        }
    }

    #[test]
    fn validate_forwards_context_to_validators() {
        let mut validable: ValidableType<usize, ValidationError, usize> =
            ValidableType::new(8, vec![Box::new(MinValidator { floor: 5 }).into()]);

        validable.validate(&2);
        assert!(validable.errors().is_empty(), "8 >= 5 + 2 should pass");

        validable.validate(&5);
        assert_eq!(
            validable.errors(),
            &[ValidationError::Range(RangeError {
                min: Some("10".to_string()),
                max: None,
                exclusive_min: None,
                exclusive_max: None,
            })],
            "8 < 5 + 5 should fail",
        );
    }
}
