use thiserror::Error;

#[derive(Debug, Error)]
pub enum LsFileStoreError {
    #[error("RepositoryNotFound: [{repository}]")]
    RepositoryNotFound { repository: String },

    #[error("InvalidPath: {field} cannot be empty")]
    InvalidPathEmpty { field: &'static str },

    #[error("InvalidPath: {field} contains NUL byte")]
    InvalidPathNul { field: &'static str },

    #[error("InvalidPath: {field} must be a relative path, got [{value}]")]
    InvalidPathAbsolute { field: &'static str, value: String },

    #[error("InvalidPath: {field} contains parent-directory traversal, got [{value}]")]
    InvalidPathTraversal { field: &'static str, value: String },

    #[error("PayloadTooLarge: file size [{actual}] exceeds save_max_size_bytes [{max}]")]
    PayloadTooLarge { actual: u64, max: u64 },

    #[error("MissingFilename: {message}")]
    MissingFilename { message: String },

    #[error("OpenDalError: {message}")]
    OpenDalError { message: String },

    #[error("ResponseBuildError: {message}")]
    ResponseBuildError { message: String },

    #[error("C3p0Error: {source:?}")]
    C3p0Error {
        #[from]
        source: c3p0::error::C3p0Error,
    },

    #[error("SqlxError: {source:?}")]
    SqlxError {
        #[from]
        source: c3p0::sqlx::Error,
    },
}
