use crate::error::SchedulerError;
use crate::scheduler::Scheduler;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use log::*;
use tokio::sync::{Mutex, RwLock};
use tokio::macros::support::Future;

pub struct JobScheduler {
    pub job: Job,
    schedule: Mutex<Scheduler>,
    timezone: Option<Tz>,
    next_run_at: Mutex<Option<DateTime<Utc>>>,
    last_run_at: Mutex<Option<DateTime<Utc>>>,
}

impl JobScheduler {
    pub fn new(mut schedule: Scheduler, timezone: Option<Tz>, job: Job) -> Self {
        // Determine the next time it should run
        let next_run_at = schedule.next(&Utc::now(), timezone);
        JobScheduler {
            job,
            schedule: Mutex::new(schedule),
            timezone,
            next_run_at: Mutex::new(next_run_at),
            last_run_at: Mutex::new(None),
        }
    }

    /// Returns true if this job is pending execution.
    pub async fn is_pending(&self) -> bool {
        // Check if paused
        if !self.job.is_active {
            return false;
        }

        // Check if NOW is on or after next_run_at
        if let Some(next_run_at) = self.next_run_at.lock().await.as_ref() {
            *next_run_at < Utc::now()
        } else {
            false
        }
    }

    /// Run the job immediately and re-schedule it.
    pub async fn run(&self) -> Result<(), SchedulerError> {
        // Execute the job function
        let run_result = self.job.run().await;

        let now = Utc::now();

        let mut schedule = self.schedule.lock().await;

        // Determine the next time it should run
        let mut next_run_at = self.next_run_at.lock().await;
        *next_run_at = schedule.next(&now, self.timezone);

        // Save the last time this ran
        let mut last_run_at = self.last_run_at.lock().await;
        *last_run_at = Some(now);

        run_result
    }
}

pub type JobFn = Box<dyn Send + Sync + Fn() -> Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + Sync + Unpin>>;

pub struct Job {
    function: Mutex<JobFn>,
    group: String,
    name: String,
    is_active: bool,
    is_running: RwLock<bool>,
    retries_after_failure: Option<u64>,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
    >(
        group: G,
        name: N,
        retries_after_failure: Option<u64>,
        function: JobFn,
    ) -> Self
    {
        Job {
            function: Mutex::new(function),
            name: name.into(),
            group: group.into(),
            retries_after_failure,
            is_running: RwLock::new(false),
            is_active: true,
        }
    }

    /// Returns true if this job is currently running.
    pub async fn is_running(&self) -> bool {
        let read = self.is_running.read().await;
        *read
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    /// Run the job immediately and re-schedule it.
    pub async fn run(&self) -> Result<(), SchedulerError> {
        self.set_running(true).await?;

        // Execute the job function
        let mut run_result = self.exec().await;

        if let Some(retries) = self.retries_after_failure {
            for attempt in 1..=retries {
                if let Err(e) = &run_result {
                    warn!(
                        "Execution failed for job [{}/{}] - Retry execution, attempt {}/{}. Previous err: {}",
                        self.group, self.name, attempt, retries, e
                    );
                    run_result = self.exec().await;
                } else {
                    break;
                }
            }
        }

        self.set_running(false).await?;

        run_result.map_err(|err| SchedulerError::JobExecutionError{ cause: err} )
    }

    async fn exec(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let function = self.function.lock().await;
        (function)().await
    }

    async fn set_running(&self, is_running: bool) -> Result<(), SchedulerError> {
        let mut write = self.is_running.write().await;

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
    use chrono_tz::UTC;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn should_be_running() {
        let lock = Arc::new(Mutex::new(true));
        let lock_clone = lock.clone();
        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        let job_scheduler = Arc::new(JobScheduler::new(
            Scheduler::Interval {
                interval_duration: Duration::new(1, 0),
                execute_at_startup: false,
            },
            Some(UTC),
            Job::new("g", "n", None, move || {
                println!("job - started");
                tx_clone.send("").unwrap();
                println!("job - Trying to get the lock");
                let _lock = lock_clone.lock();
                println!("job - lock acquired");
                Ok(())
            }),
        ));

        assert!(!job_scheduler.job.is_running());

        {
            let _lock = lock.lock();
            let job_clone = job_scheduler.clone();
            std::thread::spawn(move || {
                println!("starting job");
                job_clone.run().unwrap();
                println!("end job execution");
                tx.send("").unwrap();
            });
            rx.recv().unwrap();
            assert!(job_scheduler.job.is_running());
        }

        rx.recv().unwrap();
        assert!(!job_scheduler.job.is_running());
    }

    #[test]
    fn job_should_not_retry_run_if_ok() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            println!("job - started");
            println!("job - Trying to get the lock");
            let mut lock = lock_clone.lock();
            let count = *lock;
            *lock = count + 1;
            println!("job - count {}", count);
            Ok(())
        });

        let result = job.run();

        assert!(result.is_ok());

        let lock = lock.lock();
        let count = *lock;
        assert_eq!(1, count);
    }

    #[test]
    fn job_should_retry_run_if_error() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            println!("job - started");
            println!("job - Trying to get the lock");
            let mut lock = lock_clone.lock();
            let count = *lock;
            *lock = count + 1;
            println!("job - count {}", count);
            Err(SchedulerError::JobLockError {
                message: "".to_owned(),
            })?
        });

        let result = job.run();

        assert!(result.is_err());

        let lock = lock.lock();
        let count = *lock;
        assert_eq!(max_retries + 1, count);
    }

    #[test]
    fn job_should_stop_retrying_run_if_attempt_succeed() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let succeed_at = 7;
        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            println!("job - started");
            println!("job - Trying to get the lock");
            let mut lock = lock_clone.lock();
            let count = *lock;
            *lock = count + 1;
            println!("job - count {}", count);

            if count == succeed_at {
                Ok(())
            } else {
                Err(SchedulerError::JobLockError {
                    message: "".to_owned(),
                })?
            }
        });

        let result = job.run();

        assert!(result.is_ok());

        let lock = lock.lock();
        let count = *lock;
        assert_eq!(succeed_at + 1, count);
    }
}
