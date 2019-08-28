use crate::error::SchedulerError;
use crate::scheduler::Scheduler;
use chrono::{DateTime, TimeZone, Utc};
use std::convert::TryInto;
use std::sync::{Mutex, RwLock};

pub struct JobScheduler<T: TimeZone + Send + Sync + 'static>
where
    <T as chrono::offset::TimeZone>::Offset: std::marker::Send,
{
    pub job: Job,
    schedule: Scheduler,
    timezone: T,
    next_run_at: Mutex<Option<DateTime<T>>>,
    last_run_at: Mutex<Option<DateTime<T>>>,
}

impl<T: TimeZone + Send + Sync + 'static> JobScheduler<T>
where
    <T as chrono::offset::TimeZone>::Offset: std::marker::Send,
{
    pub fn new<S: TryInto<Scheduler>>(
        schedule: S,
        timezone: T,
        job: Job,
    ) -> Result<Self, SchedulerError>
    where
        SchedulerError: std::convert::From<<S as std::convert::TryInto<Scheduler>>::Error>,
    {
        // Determine the next time it should run
        let schedule = schedule.try_into()?;
        let next_run_at = schedule.next(&Utc::now().with_timezone(&timezone));

        Ok(JobScheduler {
            job,
            schedule,
            timezone,
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
            *next_run_at < Utc::now().with_timezone(&self.timezone)
        } else {
            false
        }
    }

    /// Run the job immediately and re-schedule it.
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Execute the job function
        let run_result = self.job.run();

        let now = Utc::now().with_timezone(&self.timezone);

        // Determine the next time it should run
        let mut next_run_at = self.next_run_at.lock().unwrap();
        *next_run_at = self.schedule.next(&now);

        // Save the last time this ran
        let mut last_run_at = self.last_run_at.lock().unwrap();
        *last_run_at = Some(now);

        run_result
    }
}

pub type JobFn = dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send;

pub struct Job {
    function: Mutex<Box<JobFn>>,
    group: String,
    name: String,
    is_active: bool,
    is_running: RwLock<bool>,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
        F: Fn() -> Result<(), Box<dyn std::error::Error>> + Send,
    >(
        group: G,
        name: N,
        function: F,
    ) -> Self
    where
        F: 'static,
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
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
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
            JobScheduler::new(
                Duration::new(1, 0),
                Utc,
                Job::new("g", "n", move || {
                    println!("job - started");
                    tx_clone.send("").unwrap();
                    println!("job - Trying to get the lock");
                    let _lock = lock_clone.lock().unwrap();
                    println!("job - lock acquired");
                    Ok(())
                }),
            )
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
