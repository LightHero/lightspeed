use thiserror::Error;

#[derive(Debug, Error)]
pub enum LsEmailError {
    #[error("ConfigurationError: {message}")]
    ConfigurationError { message: String },

    #[error("BuildTransportError: {message}")]
    BuildTransportError { message: String },

    #[error("InvalidMailbox: cannot parse [{address}]: {message}")]
    InvalidMailbox { address: String, message: String },

    #[error("InvalidMimeType: cannot parse [{mime_type}]: {message}")]
    InvalidMimeType { mime_type: String, message: String },

    #[error("AttachmentReadError: cannot read [{path}]: {message}")]
    AttachmentReadError { path: String, message: String },

    #[error("BuildMessageError: {message}")]
    BuildMessageError { message: String },

    #[error("SendError: {message}")]
    SendError { message: String },

    #[error("OperationNotSupported: {operation}")]
    OperationNotSupported { operation: &'static str },
}
