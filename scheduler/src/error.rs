use err_derive::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error(display = "ScheduleDefinitionError: [{}]", message)]
    ScheduleDefinitionError { message: String },
}
