use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("ScheduleDefinitionError: [{message}]")]
    ScheduleDefinitionError { message: String },

    #[error("JobLockError: [{message}]")]
    JobLockError { message: String },

    #[error("JobExecutionStateError: [{message}]")]
    JobExecutionStateError { message: String },

    #[error("JobExecutionError")]
    JobExecutionError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[cfg(feature = "c3p0")]
    #[error("MigrateError: {source:?}")]
    MigrateError {
        #[from]
        source: c3p0::sqlx::migrate::MigrateError,
    },

    #[cfg(feature = "c3p0")]
    #[error("SqlxError: {source:?}")]
    SqlxError {
        #[from]
        source: c3p0::sqlx::Error,
    },
}

#[cfg(feature = "c3p0")]
impl From<c3p0::C3p0Error> for SchedulerError {
    fn from(e: c3p0::C3p0Error) -> Self {
        match e {
            c3p0::C3p0Error::SqlxError(e) => SchedulerError::SqlxError { source: e },
            c3p0::C3p0Error::OptimisticLockError { cause } => SchedulerError::JobLockError { message: cause },
            c3p0::C3p0Error::Other { cause } => SchedulerError::JobExecutionStateError { message: cause },
        }
    }
}
