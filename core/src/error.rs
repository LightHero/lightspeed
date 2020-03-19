use c3p0_common::error::C3p0Error;
use serde_derive::Serialize;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use thiserror::Error;
use typescript_definitions::TypeScriptify;

#[derive(Error, Debug)]
pub enum LightSpeedError {
    // JWT
    #[error("InvalidTokenError: [{message}]")]
    InvalidTokenError { message: String },
    #[error("ExpiredTokenError: [{message}]")]
    ExpiredTokenError { message: String },
    #[error("GenerateTokenError: [{message}]")]
    GenerateTokenError { message: String },
    #[error("MissingAuthTokenError")]
    MissingAuthTokenError,
    #[error("ParseAuthHeaderError: [{message}]")]
    ParseAuthHeaderError { message: String },

    // Module
    #[error("ModuleBuilderError: [{message}]")]
    ModuleBuilderError { message: String },
    #[error("ModuleStartError: [{message}]")]
    ModuleStartError { message: String },
    #[error("ConfigurationError: [{message}]")]
    ConfigurationError { message: String },

    // Auth
    #[error("UnauthenticatedError")]
    UnauthenticatedError,
    #[error("ForbiddenError [{message}]")]
    ForbiddenError { message: String },
    #[error("PasswordEncryptionError [{message}]")]
    PasswordEncryptionError { message: String },

    #[error("InternalServerError [{message}]")]
    InternalServerError { message: String },

    #[error("RepositoryError [{message}]")]
    RepositoryError { message: String },

    #[error("ValidationError [{details:?}]")]
    ValidationError { details: RootErrorDetails },

    #[error("BadRequest [{message}]")]
    BadRequest { message: String },

    #[error("RequestConflict [{message}]")]
    RequestConflict { message: String, code: &'static str },

    #[error("ServiceUnavailable [{message}]")]
    ServiceUnavailable { message: String, code: &'static str },
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, TypeScriptify)]
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

impl From<(&str, Vec<String>)> for ErrorDetail {
    fn from(error: (&str, Vec<String>)) -> Self {
        ErrorDetail {
            error: error.0.to_string(),
            params: error.1,
        }
    }
}

#[derive(Serialize, TypeScriptify)]
pub struct WebErrorDetails<'a> {
    pub code: u16,
    pub message: &'a Option<String>,
    pub details: Option<&'a HashMap<String, Vec<ErrorDetail>>>,
}

impl<'a> WebErrorDetails<'a> {
    pub fn from_message(code: u16, message: &'a Option<String>) -> Self {
        WebErrorDetails {
            code,
            message,
            details: None,
        }
    }

    pub fn from_error_details(code: u16, error_details: &'a RootErrorDetails) -> Self {
        WebErrorDetails {
            code,
            message: &error_details.message,
            details: Some(&error_details.details),
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

impl From<C3p0Error> for LightSpeedError {
    fn from(err: C3p0Error) -> Self {
        LightSpeedError::RepositoryError {
            message: format!("{}", err),
        }
    }
}

pub trait ErrorDetails {
    fn add_detail(&mut self, key: String, value: ErrorDetail);
    fn with_scope(&mut self, scope: String) -> ScopedErrorDetails<'_>;
}

#[derive(Debug)]
pub struct RootErrorDetails {
    pub message: Option<String>,
    pub details: HashMap<String, Vec<ErrorDetail>>,
}

impl Default for RootErrorDetails {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ScopedErrorDetails<'a> {
    scope: String,
    details: &'a mut RootErrorDetails,
}

impl RootErrorDetails {
    pub fn new() -> Self {
        RootErrorDetails {
            message: None,
            details: HashMap::new(),
        }
    }

    pub fn add_into_detail<K: Into<String>, V: Into<ErrorDetail>>(&mut self, key: K, value: V) {
        self.add_detail(key.into(), value.into())
    }
}

impl ErrorDetails for RootErrorDetails {
    fn add_detail(&mut self, key: String, value: ErrorDetail) {
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
        ScopedErrorDetails {
            scope: scope.into(),
            details: self,
        }
    }
}

impl<'a> ScopedErrorDetails<'a> {
    pub fn add_into_detail<K: Into<String>, V: Into<ErrorDetail>>(&mut self, key: K, value: V) {
        self.details.add_detail(key.into(), value.into())
    }
}

impl<'a> ErrorDetails for ScopedErrorDetails<'a> {
    fn add_detail(&mut self, key: String, value: ErrorDetail) {
        let scoped_key = format!("{}.{}", self.scope, key);
        self.details.add_detail(scoped_key, value)
    }

    fn with_scope(&mut self, scope: String) -> ScopedErrorDetails<'_> {
        ScopedErrorDetails {
            scope: format!("{}.{}", self.scope, scope),
            details: self.details,
        }
    }
}

impl From<serde_json::Error> for LightSpeedError {
    fn from(err: serde_json::Error) -> Self {
        LightSpeedError::BadRequest {
            message: format!("{}", err),
        }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn error_details_should_add_entries() {
        let mut err = RootErrorDetails::new();
        assert!(err.details.is_empty());

        err.add_detail("hello".into(), "world_1".into());
        err.add_detail("hello".into(), "world_2".into());
        err.add_detail("baby".into(), "asta la vista".into());

        assert_eq!(2, err.details.len());
        assert_eq!(
            vec!["world_1".to_owned(), "world_2".to_owned()],
            err.details["hello"]
        );
        assert_eq!(vec!["asta la vista".to_owned()], err.details["baby"]);
    }

    #[test]
    pub fn error_details_should_have_scoped_children() {
        let mut root = RootErrorDetails::new();

        root.add_detail("root".into(), "world_1".into());

        {
            let mut child_one = root.with_scope("one".into());
            child_one.add_detail("A".into(), "child one.A".into());
            child_one.add_detail("B".into(), "child one.B".into());
            {
                let mut child_one_one = child_one.with_scope("inner".into());
                child_one_one.add_detail("A".into(), "child one.inner.A".into());
            }
        }

        {
            let mut child_two = root.with_scope("two".into());
            child_two.add_detail("A".into(), "child two.A".into());
        }

        use_validator(&mut root.with_scope("some".into()), "", "");
        use_validator(&mut root, "", "");
        use_validator(&mut root, "", "");

        assert_eq!(5, root.details.len());

        println!("{:?}", root.details);

        assert_eq!(ErrorDetail::new("world_1", vec![]), root.details["root"][0]);
        assert_eq!(
            ErrorDetail::new("child one.A", vec![]),
            root.details["one.A"][0]
        );
        assert_eq!(
            ErrorDetail::new("child one.B", vec![]),
            root.details["one.B"][0]
        );
        assert_eq!(
            ErrorDetail::new("child one.inner.A", vec![]),
            root.details["one.inner.A"][0]
        );
        assert_eq!(
            ErrorDetail::new("child two.A", vec![]),
            root.details["two.A"][0]
        );
    }

    fn use_validator<E: ErrorDetails>(_error_details: &mut E, _field_name: &str, _err: &str) {}
}
