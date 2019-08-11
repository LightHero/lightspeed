use crate::schedule::Schedule;
use chrono::{DateTime, Utc};
use std::sync::{Mutex, RwLock};
use crate::error::SchedulerError;
use std::convert::TryInto;

pub struct Job {
    function: Mutex<Box<Fn() -> Result<(), Box<std::error::Error>> + Send>>,
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
        S: TryInto<Schedule>,
        F: Fn() -> Result<(), Box<std::error::Error>> + Send,
    >(
        group: G,
        name: N,
        schedule: S,
        function: F,
    ) -> Result<Self, SchedulerError>
    where
        F: 'static,
        SchedulerError: std::convert::From<<S as std::convert::TryInto<Schedule>>::Error>
    {
        Ok(Job {
            function: Mutex::new(Box::new(function)),
            name: name.into(),
            group: group.into(),
            schedule: schedule.try_into()?,
            next_run_at: Mutex::new(None),
            last_run_at: Mutex::new(None),
            is_running: RwLock::new(false),
            is_active: true,
        })
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

#[cfg(test)]
pub mod test {

    use super::*;
    use std::time::Duration;
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    #[test]
    fn should_be_running() {

        let lock = Arc::new(Mutex::new(true));
        let lock_clone = lock.clone();
        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        let job = Arc::new(Job::new("g", "n", Duration::new(1,0),
            move || {
                println!("job - started");
                tx_clone.send("").unwrap();
                println!("job - Trying to get the lock");
                let _lock = lock_clone.lock().unwrap();
                println!("job - lock acquired");
                Ok(())
            }).unwrap());

        assert!(!job.is_running().unwrap());
        {
            let _lock = lock.lock().unwrap();
            let job_clone = job.clone();
            std::thread::spawn(move || {
                println!("starting job");
                job_clone.run().unwrap();
                println!("end job execution");
                tx.send("").unwrap();
            });
            rx.recv().unwrap();
            assert!(job.is_running().unwrap());
        }
        rx.recv().unwrap();
        assert!(!job.is_running().unwrap());
    }
}