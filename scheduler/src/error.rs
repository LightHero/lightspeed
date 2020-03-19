use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("ScheduleDefinitionError: [{message}]")]
    ScheduleDefinitionError { message: String },
    #[error("JobLockError: [{message}]")]
    JobLockError { message: String },
}
