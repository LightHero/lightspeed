use c3p0_common::error::C3p0Error;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct ErrorCodes {}

impl ErrorCodes {
    pub const ACTIVE_USER: &'static str = "ACTIVE_USER";
    pub const INACTIVE_USER: &'static str = "INACTIVE_USER";
    pub const INCOMPLETE_REQUEST: &'static str = "INCOMPLETE_REQUEST";
    pub const IO_ERROR: &'static str = "IO_ERROR";
    pub const JSON_PARSE_ERROR: &'static str = "JSON_PARSE_ERROR";
    pub const NOT_FOUND: &'static str = "NOT_FOUND";
    pub const NOT_PENDING_USER: &'static str = "NOT_PENDING_USER";
    pub const PARSE_ERROR: &'static str = "PARSE_ERROR";
    pub const WRONG_CREDENTIALS: &'static str = "WRONG_CREDENTIALS";
}

#[derive(Debug)]
pub enum LightSpeedError {
    InvalidTokenError { message: String },
    ExpiredTokenError { message: String },
    GenerateTokenError { message: String },
    MissingAuthTokenError,
    ParseAuthHeaderError { message: String },

    // Module
    ModuleBuilderError { message: String },
    ModuleStartError { message: String },
    ConfigurationError { message: String },

    // Auth
    UnauthenticatedError,
    ForbiddenError { message: String },
    PasswordEncryptionError { message: String },

    InternalServerError { message: String },

    C3p0Error { source: C3p0Error },

    ValidationError { details: RootErrorDetails },

    BadRequest { message: String, code: &'static str },

    RequestConflict { message: String, code: &'static str },

    ServiceUnavailable { message: String, code: &'static str },
}

impl Display for LightSpeedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LightSpeedError::InvalidTokenError { message } => write!(f, "InvalidTokenError: [{}]", message),
            LightSpeedError::ExpiredTokenError { message } => write!(f, "ExpiredTokenError: [{}]", message),
            LightSpeedError::GenerateTokenError { message } => write!(f, "GenerateTokenError: [{}]", message),
            LightSpeedError::MissingAuthTokenError => write!(f, "MissingAuthTokenError"),
            LightSpeedError::ParseAuthHeaderError { message } => write!(f, "ParseAuthHeaderError: [{}]", message),

            // Module
            LightSpeedError::ModuleBuilderError { message } => write!(f, "ModuleBuilderError: [{}]", message),
            LightSpeedError::ModuleStartError { message } => write!(f, "ModuleStartError: [{}]", message),
            LightSpeedError::ConfigurationError { message } => write!(f, "ConfigurationError: [{}]", message),

            // Auth
            LightSpeedError::UnauthenticatedError => write!(f, "UnauthenticatedError"),
            LightSpeedError::ForbiddenError { message } => write!(f, "ForbiddenError: [{}]", message),
            LightSpeedError::PasswordEncryptionError { message } => write!(f, "PasswordEncryptionError: [{}]", message),

            LightSpeedError::InternalServerError { message } => write!(f, "InternalServerError: [{}]", message),

            LightSpeedError::C3p0Error { .. } => write!(f, "C3p0Error"),

            LightSpeedError::ValidationError { details } => write!(f, "ValidationError: [{:?}]", details),

            LightSpeedError::BadRequest { message, code } => {
                write!(f, "BadRequest. Code [{}]. Message [{}]", code, message)
            }

            LightSpeedError::RequestConflict { message, code } => {
                write!(f, "RequestConflict. Code [{}]. Message [{}]", code, message)
            }

            LightSpeedError::ServiceUnavailable { message, code } => {
                write!(f, "ServiceUnavailable. Code [{}]. Message [{}]", code, message)
            }
        }
    }
}

impl Error for LightSpeedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LightSpeedError::InvalidTokenError { .. } |
            LightSpeedError::ExpiredTokenError{ .. } |
            LightSpeedError::GenerateTokenError { .. } |
            LightSpeedError::MissingAuthTokenError { .. } |
            LightSpeedError::ParseAuthHeaderError { .. } |

            LightSpeedError::ModuleBuilderError { .. } |
            LightSpeedError::ModuleStartError { .. } |
            LightSpeedError::ConfigurationError { .. } |

            // Auth
            LightSpeedError::UnauthenticatedError { .. } |
            LightSpeedError::ForbiddenError { .. } |
            LightSpeedError::PasswordEncryptionError { .. } |

            LightSpeedError::InternalServerError { .. } |


            LightSpeedError::ValidationError { .. } |

            LightSpeedError::BadRequest { .. } |

            LightSpeedError::RequestConflict { .. } |
            LightSpeedError::ServiceUnavailable { .. } => None,

            LightSpeedError::C3p0Error { source } => Some(source),
        }
    }
}

impl From<c3p0_common::error::C3p0Error> for LightSpeedError {
    fn from(err: c3p0_common::error::C3p0Error) -> Self {
        LightSpeedError::C3p0Error { source: err }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub error: String,
    pub params: Vec<String>,
}

impl ErrorDetail {
    pub fn new<S: Into<String>>(error: S, params: Vec<String>) -> Self {
        ErrorDetail { error: error.into(), params }
    }
}

impl From<String> for ErrorDetail {
    fn from(error: String) -> Self {
        ErrorDetail { error, params: vec![] }
    }
}

impl From<&str> for ErrorDetail {
    fn from(error: &str) -> Self {
        ErrorDetail { error: error.to_string(), params: vec![] }
    }
}

impl From<(&str, Vec<String>)> for ErrorDetail {
    fn from(error: (&str, Vec<String>)) -> Self {
        ErrorDetail { error: error.0.to_string(), params: error.1 }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WebErrorDetails<'a> {
    pub code: u16,
    pub message: Option<Cow<'a, str>>,
    pub details: Option<Cow<'a, HashMap<String, Vec<ErrorDetail>>>>,
}

impl<'a> WebErrorDetails<'a> {
    pub fn from_message(code: u16, message: Option<Cow<'a, str>>) -> Self {
        WebErrorDetails { code, message, details: None }
    }

    pub fn from_error_details(code: u16, error_details: &'a RootErrorDetails) -> Self {
        WebErrorDetails {
            code,
            message: error_details.message.as_ref().map(|val| val.into()),
            details: Some(Cow::Borrowed(&error_details.details)),
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

pub type ErrorDetailsData = HashMap<String, Vec<ErrorDetail>>;

pub enum ErrorDetails<'a> {
    Root(RootErrorDetails),
    Scoped(ScopedErrorDetails<'a>),
}

impl<'a> Default for ErrorDetails<'a> {
    fn default() -> Self {
        ErrorDetails::Root(Default::default())
    }
}

impl<'a> ErrorDetails<'a> {
    pub fn add_detail<K: Into<String>, V: Into<ErrorDetail>>(&mut self, key: K, value: V) {
        match self {
            ErrorDetails::Root(node) => node.add_detail(key.into(), value.into()),
            ErrorDetails::Scoped(node) => node.add_detail(key.into(), value.into()),
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

#[derive(Debug)]
pub struct RootErrorDetails {
    pub message: Option<String>,
    pub details: HashMap<String, Vec<ErrorDetail>>,
}

impl Default for RootErrorDetails {
    fn default() -> Self {
        RootErrorDetails { message: None, details: HashMap::new() }
    }
}

#[derive(Debug)]
pub struct ScopedErrorDetails<'a> {
    scope: String,
    details: &'a mut RootErrorDetails,
}

impl RootErrorDetails {
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
        ScopedErrorDetails { scope, details: self }
    }
}

impl<'a> ScopedErrorDetails<'a> {
    fn add_detail(&mut self, key: String, value: ErrorDetail) {
        let scoped_key = format!("{}.{}", self.scope, key);
        self.details.add_detail(scoped_key, value)
    }

    fn with_scope(&mut self, scope: String) -> ScopedErrorDetails<'_> {
        ScopedErrorDetails { scope: format!("{}.{}", self.scope, scope), details: self.details }
    }
}

impl From<serde_json::Error> for LightSpeedError {
    fn from(err: serde_json::Error) -> Self {
        LightSpeedError::BadRequest { message: format!("{:?}", err), code: ErrorCodes::JSON_PARSE_ERROR }
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn error_details_should_add_entries() {
        let mut err = ErrorDetails::default();
        assert!(err.details().is_empty());

        err.add_detail("hello", "world_1");
        err.add_detail("hello", "world_2");
        err.add_detail("baby", "asta la vista");

        assert_eq!(2, err.details().len());
        assert_eq!(vec!["world_1".to_owned(), "world_2".to_owned()], err.details()["hello"]);
        assert_eq!(vec!["asta la vista".to_owned()], err.details()["baby"]);
    }

    #[test]
    pub fn error_details_should_have_scoped_children() {
        let mut root = ErrorDetails::default();

        root.add_detail("root", "world_1");

        {
            let mut child_one = root.with_scope("one");
            child_one.add_detail("A", "child one.A");
            child_one.add_detail("B", "child one.B");
            {
                let mut child_one_one = child_one.with_scope("inner");
                child_one_one.add_detail("A", "child one.inner.A");
            }
        }

        {
            let mut child_two = root.with_scope("two");
            child_two.add_detail("A", "child two.A");
        }

        use_validator(&mut root.with_scope("some"), "", "");
        use_validator(&mut root, "", "");
        use_validator(&mut root, "", "");

        assert_eq!(5, root.details().len());

        println!("{:?}", root.details());

        assert_eq!(ErrorDetail::new("world_1", vec![]), root.details()["root"][0]);
        assert_eq!(ErrorDetail::new("child one.A", vec![]), root.details()["one.A"][0]);
        assert_eq!(ErrorDetail::new("child one.B", vec![]), root.details()["one.B"][0]);
        assert_eq!(ErrorDetail::new("child one.inner.A", vec![]), root.details()["one.inner.A"][0]);
        assert_eq!(ErrorDetail::new("child two.A", vec![]), root.details()["two.A"][0]);
    }

    fn use_validator(_error_details: &mut ErrorDetails, _field_name: &str, _err: &str) {}
}
