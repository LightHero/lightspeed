use c3p0_common::error::C3p0Error;
use err_derive::Error;
use serde::Serialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum LightSpeedError {
    // JWT
    #[error(display = "InvalidTokenError: [{}]", message)]
    InvalidTokenError { message: String },
    #[error(display = "ExpiredTokenError: [{}]", message)]
    ExpiredTokenError { message: String },
    #[error(display = "GenerateTokenError: [{}]", message)]
    GenerateTokenError { message: String },
    #[error(display = "MissingAuthTokenError")]
    MissingAuthTokenError,
    #[error(display = "ParseAuthHeaderError: [{}]", message)]
    ParseAuthHeaderError { message: String },

    // Module
    #[error(display = "ModuleBuilderError: [{}]", message)]
    ModuleBuilderError { message: String },
    #[error(display = "ModuleStartError: [{}]", message)]
    ModuleStartError { message: String },
    #[error(display = "ConfigurationError: [{}]", message)]
    ConfigurationError { message: String },

    // Auth
    #[error(display = "UnauthenticatedError")]
    UnauthenticatedError,
    #[error(display = "ForbiddenError [{}]", message)]
    ForbiddenError { message: String },
    #[error(display = "PasswordEncryptionError [{}]", message)]
    PasswordEncryptionError { message: String },

    #[error(display = "InternalServerError [{}]", message)]
    InternalServerError { message: String },

    #[error(display = "RepositoryError [{}]", message)]
    RepositoryError { message: String },

    #[error(display = "ValidationError [{:?}]", details)]
    ValidationError { details: ErrorDetails },
    #[error(display = "BadRequest [{}]", message)]
    BadRequest { message: String },
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct ErrorDetails {
    pub message: Option<String>,
    pub details: HashMap<String, Vec<ErrorDetail>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct ErrorDetail {
    error: String,
    params: Vec<String>,
}

impl ErrorDetail {
    pub fn new<S: Into<String>>(error: S, params: Vec<String>) -> Self {
        ErrorDetail {
            error: error.into(),
            params,
        }
    }
}

impl From<String> for ErrorDetail {
    fn from(error: String) -> Self {
        ErrorDetail {
            error,
            params: vec![],
        }
    }
}

impl From<&str> for ErrorDetail {
    fn from(error: &str) -> Self {
        ErrorDetail {
            error: error.to_string(),
            params: vec![],
        }
    }
}

impl PartialEq<ErrorDetail> for &str {
    fn eq(&self, other: &ErrorDetail) -> bool {
        other.params.is_empty() && other.error.eq(self)
    }
}

impl PartialEq<ErrorDetail> for String {
    fn eq(&self, other: &ErrorDetail) -> bool {
        other.params.is_empty() && other.error.eq(self)
    }
}

impl ErrorDetails {
    pub fn add_detail<K: Into<String>, V: Into<ErrorDetail>>(&mut self, key: K, value: V) {
        match self.details.entry(key.into()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(value.into());
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![value.into()]);
            }
        }
    }
}

impl From<C3p0Error> for LightSpeedError {
    fn from(err: C3p0Error) -> Self {
        LightSpeedError::RepositoryError {
            message: format!("{}", err),
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn error_details_should_add_entries() {
        let mut err = ErrorDetails::default();
        assert!(err.details.is_empty());

        err.add_detail("hello", "world_1");
        err.add_detail("hello", "world_2");
        err.add_detail("baby", "asta la vista");

        assert_eq!(2, err.details.len());
        assert_eq!(
            vec!["world_1".to_owned(), "world_2".to_owned()],
            err.details["hello"]
        );
        assert_eq!(vec!["asta la vista".to_owned()], err.details["baby"]);
    }

}
