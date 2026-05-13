use std::collections::HashMap;
use std::collections::hash_map::Entry;
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ValidationError {
    #[error("MUST_BE_TRUE")]
    MustBeTrue,
    #[error("MUST_BE_FALSE")]
    MustBeFalse,
    #[error("MUST_CONTAIN")]
    MustContain { needle: String },
    #[error("NOT_VALID_EMAIL")]
    NotValidEmail,
    #[error("NOT_VALID_IP")]
    NotValidIp,
    #[error("NOT_EQUALS")]
    NotEquals { a: String, b: String },
    #[error("MUST_BE_LESS_OR_EQUAL")]
    MustBeLessOrEqual { max: String },
    #[error("MUST_BE_LESS")]
    MustBeLess { max: String },
    #[error("MUST_BE_GREATER_OR_EQUAL")]
    MustBeGreaterOrEqual { min: String },
    #[error("MUST_BE_GREATER")]
    MustBeGreater { min: String },
    #[error("NOT_VALID_URL")]
    NotValidUrl,
    #[error("WRONG_OWNER")]
    WrongOwner,
    #[error("WRONG_ID")]
    WrongId,
    #[error("WRONG_VERSION")]
    WrongVersion,
    #[error("NOT_UNIQUE")]
    NotUnique,
    #[error("VALUE_REQUIRED")]
    ValueRequired,
    #[error("UNKNOWN_FIELD")]
    UnknownField,
    #[error("{code}")]
    Custom { code: String, params: Vec<String> },
}

pub type ErrorDetailsData = HashMap<String, Vec<ValidationError>>;

pub enum ErrorDetails<'a> {
    Root(RootErrorDetails),
    Scoped(ScopedErrorDetails<'a>),
}

impl Default for ErrorDetails<'_> {
    fn default() -> Self {
        ErrorDetails::Root(Default::default())
    }
}

impl ErrorDetails<'_> {
    pub fn add_detail<K: Into<String>>(&mut self, key: K, value: ValidationError) {
        match self {
            ErrorDetails::Root(node) => node.add_detail(key.into(), value),
            ErrorDetails::Scoped(node) => node.add_detail(key.into(), value),
        }
    }

    pub fn with_scope<S: Into<String>>(&mut self, scope: S) -> ErrorDetails<'_> {
        match self {
            ErrorDetails::Root(node) => ErrorDetails::Scoped(node.with_scope(scope.into())),
            ErrorDetails::Scoped(node) => ErrorDetails::Scoped(node.with_scope(scope.into())),
        }
    }

    pub fn details(&self) -> &ErrorDetailsData {
        match self {
            ErrorDetails::Root(node) => &node.details,
            ErrorDetails::Scoped(node) => &node.details.details,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RootErrorDetails {
    pub details: ErrorDetailsData,
}

#[derive(Debug)]
pub struct ScopedErrorDetails<'a> {
    scope: String,
    details: &'a mut RootErrorDetails,
}

impl RootErrorDetails {
    fn add_detail(&mut self, key: String, value: ValidationError) {
        match self.details.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![value]);
            }
        }
    }

    fn with_scope(&mut self, scope: String) -> ScopedErrorDetails<'_> {
        ScopedErrorDetails { scope, details: self }
    }
}

impl ScopedErrorDetails<'_> {
    fn add_detail(&mut self, key: String, value: ValidationError) {
        let scoped_key = format!("{}.{}", self.scope, key);
        self.details.add_detail(scoped_key, value)
    }

    fn with_scope(&mut self, scope: String) -> ScopedErrorDetails<'_> {
        ScopedErrorDetails { scope: format!("{}.{}", self.scope, scope), details: self.details }
    }
}

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("ValidationFailed: {details:?}")]
    ValidationFailed { details: RootErrorDetails },
    #[error("{0}")]
    Error(Box<dyn std::error::Error + Send + Sync>),
}
