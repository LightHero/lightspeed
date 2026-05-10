use crate::error::SchedulerError;
use crate::scheduler::Scheduler;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use log::*;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

/// Wraps a [`Scheduler`] with the bookkeeping needed to fire a [`Job`].
///
/// **Missed-slot semantics.** When a job's run takes longer than its interval
/// (or spans a cron tick), the missed slots are *skipped* rather than queued
/// for catch-up. After every run, `next_run_at` is recomputed from the
/// current time via `Scheduler::next(now, …)`, so:
///
/// * `Scheduler::Interval { duration, .. }` — the next fire is `now +
///   duration`, where `now` is the moment the run completed. A 5-minute
///   interval that takes 12 minutes to run fires once after the run
///   finishes, not three times back-to-back.
/// * `Scheduler::Cron(_)` — the next fire is the first cron occurrence
///   strictly after the run completed; intermediate occurrences are dropped.
///
/// Callers that need at-least-once / catch-up semantics should layer their
/// own queue on top — this scheduler intentionally trades replay for
/// bounded resource use under load.
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

        // Check if NOW is on or after next_run_at. Use `<=` so a tick that
        // lands exactly on `next_run_at` is treated as pending — at sub-
        // second poll rates, strict `<` would jitter and skip slots whose
        // scheduled time happens to coincide with the tick instant.
        match self.next_run_at.lock().await.as_ref() {
            Some(next_run_at) => *next_run_at <= Utc::now(),
            _ => false,
        }
    }

    /// Run the job immediately and re-schedule it.
    pub async fn run(&self) -> Result<(), SchedulerError> {
        if !self.job.try_claim_running() {
            return Err(SchedulerError::JobLockError {
                message: format!(
                    "Wrong Job status found for job [{}/{}]. Expected: false",
                    self.job.group, self.job.name
                ),
            });
        }
        self.run_after_claim().await
    }

    /// Run the job assuming the running slot has already been atomically
    /// claimed by the caller via [`Job::try_claim_running`]. The slot is
    /// released on completion (or panic) by the internal Drop guard.
    pub(crate) async fn run_after_claim(&self) -> Result<(), SchedulerError> {
        let run_result = self.job.run_with_guard().await;

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

pub type JobFn = dyn 'static
    + Send
    + Sync
    + Fn() -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;

pub struct Job {
    function: Arc<JobFn>,
    group: String,
    name: String,
    is_active: bool,
    is_running: AtomicBool,
    retries_after_failure: Option<u64>,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
        F: 'static
            + Send
            + Sync
            + Fn() -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>,
    >(
        group: G,
        name: N,
        retries_after_failure: Option<u64>,
        function: F,
    ) -> Self {
        Job {
            function: Arc::new(function),
            name: name.into(),
            group: group.into(),
            retries_after_failure,
            is_running: AtomicBool::new(false),
            is_active: true,
        }
    }

    /// Returns true if this job is currently running.
    pub async fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Atomically tries to claim the running slot. Returns true if the
    /// caller now owns the slot; false if another caller already holds it.
    /// The owner must release the slot via [`Job::run_with_guard`] (which
    /// installs a Drop guard) so the flag is cleared even on panic.
    pub(crate) fn try_claim_running(&self) -> bool {
        self.is_running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    /// Run the job immediately. Claims the running slot atomically; returns
    /// `JobLockError` if it is already claimed. The slot is released on
    /// completion (or panic) via an internal Drop guard.
    pub async fn run(&self) -> Result<(), SchedulerError> {
        if !self.try_claim_running() {
            return Err(SchedulerError::JobLockError {
                message: format!("Wrong Job status found for job [{}/{}]. Expected: false", self.group, self.name),
            });
        }
        self.run_with_guard().await
    }

    /// Run the job. The caller MUST have already claimed the running slot
    /// via [`Job::try_claim_running`]. The slot is released by the Drop
    /// guard, so it is cleared even if the user-supplied future panics.
    async fn run_with_guard(&self) -> Result<(), SchedulerError> {
        let _guard = RunningGuard { flag: &self.is_running };

        // Execute the job function
        let mut run_result = self.exec().await;

        if let Some(retries) = self.retries_after_failure {
            for attempt in 1..=retries {
                match run_result {
                    Err(e) => {
                        warn!(
                            "Execution failed for job [{}/{}] - Retry execution, attempt {}/{}. Previous err: {}",
                            self.group, self.name, attempt, retries, e
                        );
                        run_result = self.exec().await;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        run_result.map_err(|err| SchedulerError::JobExecutionError { source: err })
    }

    async fn exec(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let function = self.function.clone();
        (function)().await
    }
}

struct RunningGuard<'a> {
    flag: &'a AtomicBool,
}

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use chrono_tz::UTC;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn should_be_running() {
        let lock = Arc::new(Mutex::new(true));
        let lock_clone = lock.clone();
        let (tx, mut rx) = channel(10000);
        let tx_clone = tx.clone();

        let job_scheduler = Arc::new(JobScheduler::new(
            Scheduler::Interval { interval_duration: Duration::new(1, 0), execute_at_startup: false },
            Some(UTC),
            Job::new("g", "n", None, move || {
                let lock_clone = lock_clone.clone();
                let tx_clone = tx_clone.clone();
                Box::pin(async move {
                    println!("job - started");
                    tx_clone.send("").await.unwrap();
                    println!("job - Trying to get the lock");
                    let _lock = lock_clone.lock().await;
                    println!("job - lock acquired");
                    Ok(())
                })
            }),
        ));

        assert!(!job_scheduler.job.is_running().await);

        {
            let _lock = lock.lock().await;
            let job_clone = job_scheduler.clone();
            tokio::spawn(async move {
                println!("starting job");
                job_clone.run().await.unwrap();
                println!("end job execution");
                tx.send("").await.unwrap();
            });
            rx.recv().await.unwrap();
            assert!(job_scheduler.job.is_running().await);
        }

        rx.recv().await.unwrap();
        assert!(!job_scheduler.job.is_running().await);
    }

    #[tokio::test]
    async fn job_should_not_retry_run_if_ok() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            let lock_clone = lock_clone.clone();
            Box::pin(async move {
                println!("job - started");
                println!("job - Trying to get the lock");
                let mut lock = lock_clone.lock().await;
                let count = *lock;
                *lock = count + 1;
                println!("job - count {count}");
                Ok(())
            })
        });

        let result = job.run().await;

        assert!(result.is_ok());

        let lock = lock.lock().await;
        let count = *lock;
        assert_eq!(1, count);
    }

    #[tokio::test]
    async fn job_should_retry_run_if_error() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            let lock_clone = lock_clone.clone();
            Box::pin(async move {
                println!("job - started");
                println!("job - Trying to get the lock");
                let mut lock = lock_clone.lock().await;
                let count = *lock;
                *lock = count + 1;
                println!("job - count {count}");
                Err(SchedulerError::JobLockError { message: "".to_owned() })?
            })
        });

        let result = job.run().await;

        assert!(result.is_err());

        let lock = lock.lock().await;
        let count = *lock;
        assert_eq!(max_retries + 1, count);
    }

    #[tokio::test]
    async fn job_should_stop_retrying_run_if_attempt_succeed() {
        let lock = Arc::new(Mutex::new(0));
        let lock_clone = lock.clone();

        let succeed_at = 7;
        let max_retries = 12;

        let job = Job::new("g", "n", Some(max_retries), move || {
            let lock_clone = lock_clone.clone();
            Box::pin(async move {
                println!("job - started");
                println!("job - Trying to get the lock");
                let mut lock = lock_clone.lock().await;
                let count = *lock;
                *lock = count + 1;
                println!("job - count {count}");

                if count == succeed_at { Ok(()) } else { Err(SchedulerError::JobLockError { message: "".to_owned() })? }
            })
        });

        let result = job.run().await;

        assert!(result.is_ok());

        let lock = lock.lock().await;
        let count = *lock;
        assert_eq!(succeed_at + 1, count);
    }
}
