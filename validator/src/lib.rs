
pub mod validation;
pub mod error;

pub use error::*;
pub use lightspeed_validator_derive::Validable;

pub struct ValidableType<T> {
    value: T,
    errors: Vec<ValidationError>,
}

impl<T> ValidableType<T> {
    pub fn new(value: T) -> Self {
        Self { value, errors: vec![] }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn errors(&self) -> &Vec<ValidationError> {
        &self.errors
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn push_error(&mut self, err: ValidationError) {
        self.errors.push(err);
    }
}