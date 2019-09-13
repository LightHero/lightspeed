use c3p0_common::error::C3p0Error;
use err_derive::Error;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

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

#[derive(Debug, PartialEq)]
pub enum ErrorDetails {
    Root {
        message: Option<String>,
        details: Rc<RefCell<HashMap<String, Vec<ErrorDetail>>>>,
    },
    Child {
        scope: String,
        details: Rc<RefCell<HashMap<String, Vec<ErrorDetail>>>>,
    },
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

impl From<(&str, Vec<String>)> for ErrorDetail {
    fn from(error: (&str, Vec<String>)) -> Self {
        ErrorDetail {
            error: error.0.to_string(),
            params: error.1,
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

impl Default for ErrorDetails {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorDetails {
    pub fn new() -> Self {
        ErrorDetails::Root {
            message: None,
            details: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn add_detail<K: Into<String>, V: Into<ErrorDetail>>(&self, key: K, value: V) {
        let (scoped_key, details) = match self {
            ErrorDetails::Root { details, .. } => (key.into(), details),
            ErrorDetails::Child { details, scope } => {
                (format!("{}.{}", scope, key.into()), details)
            }
        };
        match details.deref().borrow_mut().entry(scoped_key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(value.into());
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![value.into()]);
            }
        }
    }

    pub fn details(&self) -> &Rc<RefCell<HashMap<String, Vec<ErrorDetail>>>> {
        match self {
            ErrorDetails::Root { details, .. } => details,
            ErrorDetails::Child { details, .. } => details,
        }
    }

    pub fn with_scope<S: Into<String>>(&self, scope: S) -> ErrorDetails {
        let (scoped_key, details) = match self {
            ErrorDetails::Root { details, .. } => (scope.into(), details),
            ErrorDetails::Child {
                details,
                scope: this_scope,
            } => (format!("{}.{}", this_scope, scope.into()), details),
        };
        ErrorDetails::Child {
            scope: scoped_key,
            details: details.clone(),
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
        let err = ErrorDetails::new();
        assert!(err.details().borrow().is_empty());

        err.add_detail("hello", "world_1");
        err.add_detail("hello", "world_2");
        err.add_detail("baby", "asta la vista");

        assert_eq!(2, err.details().borrow().len());
        assert_eq!(
            vec!["world_1".to_owned(), "world_2".to_owned()],
            err.details().borrow()["hello"]
        );
        assert_eq!(
            vec!["asta la vista".to_owned()],
            err.details().borrow()["baby"]
        );
    }

    #[test]
    pub fn error_details_should_have_scoped_children() {
        let root = ErrorDetails::new();

        root.add_detail("root", "world_1");

        let child_one = root.with_scope("one");
        child_one.add_detail("A", "child one.A");
        child_one.add_detail("B", "child one.B");

        let child_one_one = child_one.with_scope("inner");
        child_one_one.add_detail("A", "child one.inner.A");

        let child_two = root.with_scope("two");
        child_two.add_detail("A", "child two.A");

        assert_eq!(5, root.details().borrow().len());

        println!("{:?}", root.details());

        assert_eq!(
            ErrorDetail::new("world_1", vec![]),
            root.details().clone().borrow()["root"][0]
        );
        assert_eq!(
            ErrorDetail::new("child one.A", vec![]),
            root.details().clone().borrow()["one.A"][0]
        );
        assert_eq!(
            ErrorDetail::new("child one.B", vec![]),
            root.details().clone().borrow()["one.B"][0]
        );
        assert_eq!(
            ErrorDetail::new("child one.inner.A", vec![]),
            root.details().clone().borrow()["one.inner.A"][0]
        );
        assert_eq!(
            ErrorDetail::new("child two.A", vec![]),
            root.details().clone().borrow()["two.A"][0]
        );
    }

}
