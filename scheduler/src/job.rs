use crate::schedule::Schedule;
use chrono::{DateTime, Utc};
use std::sync::{Mutex, RwLock};
use crate::error::SchedulerError;

pub struct Job {
    function: Mutex<Box<Fn() -> Result<(), Box<std::error::Error>> + Send + Sync>>,
    group: String,
    name: String,
    schedule: Schedule,
    next_run_at: Mutex<Option<DateTime<Utc>>>,
    last_run_at: Mutex<Option<DateTime<Utc>>>,
    is_active: bool,
    is_running: RwLock<bool>,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
        S: Into<Schedule>,
        F: Fn() -> Result<(), Box<std::error::Error>> + Send + Sync,
    >(
        group: G,
        name: N,
        schedule: S,
        function: F,
    ) -> Self
    where
        F: 'static,
    {
        Job {
            function: Mutex::new(Box::new(function)),
            name: name.into(),
            group: group.into(),
            schedule: schedule.into(),
            next_run_at: Mutex::new(None),
            last_run_at: Mutex::new(None),
            is_running: RwLock::new(false),
            is_active: true,
        }
    }

    /// Returns true if this job is pending execution.
    pub fn is_pending(&self) -> bool {
        // Check if paused
        if !self.is_active {
            return false;
        }

        // Check if NOW is on or after next_run_at
        if let Some(next_run_at) = self.next_run_at.lock().unwrap().as_ref() {
            &Utc::now() >= next_run_at
        } else {
            false
        }
    }

    /// Returns true if this job is currently running.
    pub fn is_running(&self) -> Result<bool, SchedulerError> {
        let read = self.is_running.read()
            .map_err(|err| SchedulerError::JobLockError { message: format!("Cannot read is_running status of job [{}/{}]. Err: {}", self.group, self.name, err)})?;
        Ok(*read)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    /// Run the job immediately and re-schedule it.
    pub fn run(&self) -> Result<(), Box<std::error::Error>> {

        self.set_running(true)?;

        // Execute the job function
        let run_result = {
            let function = self.function.lock()
                .map_err(|err| SchedulerError::JobLockError { message: format!("Cannot execute job [{}/{}]. Err: {}", self.group, self.name, err)})?;
            (function)()
        };

        let now = Some(Utc::now());

        // Determine the next time it should run
        let mut next_run_at = self.next_run_at.lock().unwrap();
        *next_run_at = self.schedule.next(&now);

        // Save the last time this ran
        let mut last_run_at = self.last_run_at.lock().unwrap();
        *last_run_at = now;

        self.set_running(false)?;

        run_result
    }

    fn set_running(&self, is_running: bool) -> Result<(), SchedulerError>{
        let mut write = self.is_running.write()
            .map_err(|err| SchedulerError::JobLockError { message: format!("Cannot write is_running status of job [{}/{}]. Err: {}", self.group, self.name, err)})?;

        if is_running.eq(&*write) {
            return Err(SchedulerError::JobLockError { message: format!("Wrong Job status found for job [{}/{}]. Expected: {}", self.group, self.name, !is_running)});
        }

        *write = is_running;
        Ok(())
    }
}
