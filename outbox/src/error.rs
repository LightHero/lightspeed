use thiserror::Error;

#[derive(Error, Debug)]
pub enum OutboxError {
    #[error("DbError: {source:?}")]
    DbError {
        #[from]
        source: c3p0::error::C3p0Error,
    },

    #[cfg(feature = "sqlx")]
    #[error("MigrateError: {source:?}")]
    MigrateError {
        #[from]
        source: sqlx::migrate::MigrateError,
    },

    #[error("SqlxError: {source:?}")]
    SqlxError {
        #[from]
        source: c3p0::sqlx::Error,
    },

    #[error("JsonError: {source:?}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
}

impl From<OutboxError> for lightspeed_core::error::LsError {
    fn from(value: OutboxError) -> Self {
        lightspeed_core::error::LsError::ExecutionError { message: format!("OutboxError: {value:?}") }
    }
}
