use crate::error::SchedulerError;
use crate::schedule::Schedule;
use chrono::{DateTime, Utc};
use std::convert::TryInto;
use std::sync::{Mutex, RwLock};

pub struct JobScheduler {
    pub job: Job,
    schedule: Schedule,
    next_run_at: Mutex<Option<DateTime<Utc>>>,
    last_run_at: Mutex<Option<DateTime<Utc>>>,
}

impl JobScheduler {
    pub fn new<
        S: TryInto<Schedule>
    >(
        schedule: S,
        job: Job,
    ) -> Result<Self, SchedulerError>
        where
            SchedulerError: std::convert::From<<S as std::convert::TryInto<Schedule>>::Error>,
    {
        // Determine the next time it should run
        let schedule = schedule.try_into()?;
        let next_run_at = schedule.next(&None);

        Ok(JobScheduler {
            job,
            schedule,
            next_run_at: Mutex::new(next_run_at),
            last_run_at: Mutex::new(None),
        })
    }

    /// Returns true if this job is pending execution.
    pub fn is_pending(&self) -> bool {
        // Check if paused
        if !self.job.is_active {
            return false;
        }

        // Check if NOW is on or after next_run_at
        if let Some(next_run_at) = self.next_run_at.lock().unwrap().as_ref() {
            &Utc::now() >= next_run_at
        } else {
            false
        }
    }

    /// Run the job immediately and re-schedule it.
    pub fn run(&self) -> Result<(), Box<std::error::Error>> {
        // Execute the job function
        let run_result = self.job.run();

        let now = Some(Utc::now());

        // Determine the next time it should run
        let mut next_run_at = self.next_run_at.lock().unwrap();
        *next_run_at = self.schedule.next(&now);

        // Save the last time this ran
        let mut last_run_at = self.last_run_at.lock().unwrap();
        *last_run_at = now;

        run_result
    }
}

pub struct Job {
    function: Mutex<Box<Fn() -> Result<(), Box<std::error::Error>> + Send>>,
    group: String,
    name: String,
    is_active: bool,
    is_running: RwLock<bool>,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
        F: Fn() -> Result<(), Box<std::error::Error>> + Send,
    >(
        group: G,
        name: N,
        function: F,
    ) -> Self
    where
        F: 'static
    {
        Job {
            function: Mutex::new(Box::new(function)),
            name: name.into(),
            group: group.into(),
            is_running: RwLock::new(false),
            is_active: true,
        }
    }

    /// Returns true if this job is currently running.
    pub fn is_running(&self) -> Result<bool, SchedulerError> {
        let read = self
            .is_running
            .read()
            .map_err(|err| SchedulerError::JobLockError {
                message: format!(
                    "Cannot read is_running status of job [{}/{}]. Err: {}",
                    self.group, self.name, err
                ),
            })?;
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
            let function = self
                .function
                .lock()
                .map_err(|err| SchedulerError::JobLockError {
                    message: format!(
                        "Cannot execute job [{}/{}]. Err: {}",
                        self.group, self.name, err
                    ),
                })?;
            (function)()
        };

        self.set_running(false)?;

        run_result
    }

    fn set_running(&self, is_running: bool) -> Result<(), SchedulerError> {
        let mut write = self
            .is_running
            .write()
            .map_err(|err| SchedulerError::JobLockError {
                message: format!(
                    "Cannot write is_running status of job [{}/{}]. Err: {}",
                    self.group, self.name, err
                ),
            })?;

        if is_running.eq(&*write) {
            return Err(SchedulerError::JobLockError {
                message: format!(
                    "Wrong Job status found for job [{}/{}]. Expected: {}",
                    self.group, self.name, !is_running
                ),
            });
        }

        *write = is_running;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn should_be_running() {
        let lock = Arc::new(Mutex::new(true));
        let lock_clone = lock.clone();
        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        let job_scheduler = Arc::new(
                JobScheduler::new(Duration::new(1, 0),
            Job::new("g", "n", move || {
                println!("job - started");
                tx_clone.send("").unwrap();
                println!("job - Trying to get the lock");
                let _lock = lock_clone.lock().unwrap();
                println!("job - lock acquired");
                Ok(())
            }))
            .unwrap(),
        );

        assert!(!job_scheduler.job.is_running().unwrap());

        {
            let _lock = lock.lock().unwrap();
            let job_clone = job_scheduler.clone();
            std::thread::spawn(move || {
                println!("starting job");
                job_clone.run().unwrap();
                println!("end job execution");
                tx.send("").unwrap();
            });
            rx.recv().unwrap();
            assert!(job_scheduler.job.is_running().unwrap());
        }

        rx.recv().unwrap();
        assert!(!job_scheduler.job.is_running().unwrap());
    }
}
