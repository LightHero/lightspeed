use crate::error::SchedulerError;
use crate::job::{Job, JobScheduler};
use crate::scheduler::{Scheduler, TryToScheduler};
use chrono::Utc;
use chrono_tz::{Tz, UTC};
use log::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

pub mod error;
pub mod job;
pub mod scheduler;

pub struct JobExecutor {
    sleep_between_checks: Duration,
    running: RwLock<bool>,
    timezone: Option<Tz>,
    jobs: Vec<Arc<JobScheduler>>,
}

/// Creates a new Executor that uses the Local time zone for the execution times evaluation.
/// For example, the cron expressions will refer to the Local time zone.
pub fn new_executor_with_local_tz() -> JobExecutor {
    new_executor_with_tz(None)
}

/// Creates a new Executor that uses the UTC time zone for the execution times evaluation.
/// For example, the cron expressions will refer to the UTC time zone.
pub fn new_executor_with_utc_tz() -> JobExecutor {
    new_executor_with_tz(Some(UTC))
}

/// Creates a new Executor that uses a custom time zone for the execution times evaluation.
/// For example, the cron expressions will refer to the specified time zone.
pub fn new_executor_with_tz(timezone: Option<Tz>) -> JobExecutor {
    JobExecutor {
        sleep_between_checks: Duration::new(1, 0),
        running: RwLock::new(false),
        timezone,
        jobs: vec![],
    }
}

impl JobExecutor {
    /// Returns true if the JobExecutor contains no jobs.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Returns the number of jobs in the JobExecutor.
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Clear the JobExecutor, removing all jobs.
    pub fn clear(&mut self) {
        self.jobs.clear()
    }

    /// Returns true if the Job Executor is running
    pub fn is_running(&self) -> bool {
        let read = self.running.read();
        *read
    }

    /// Returns true if there is at least one job pending.
    pub fn is_pending_job(&self) -> bool {
        for job_scheduler in &self.jobs {
            if job_scheduler.is_pending() {
                return true;
            }
        }
        false
    }

    /// Returns true if there is at least one job running.
    pub fn is_running_job(&self) -> bool {
        for job_scheduler in &self.jobs {
            if job_scheduler.job.is_running() {
                return true;
            }
        }
        false
    }

    /// Run pending jobs in the JobExecutor.
    fn run_pending_jobs(&self) {
        trace!("Check pending jobs");
        for job_scheduler in &self.jobs {
            //println!("check JOB IS PENDING: {}", job.is_pending());
            if job_scheduler.is_pending() {
                //println!("JOB IS RUNNING? {}", is_running);
                if !job_scheduler.job.is_running() {
                    let job_clone = job_scheduler.clone();
                    std::thread::spawn(move || {
                        let timestamp = Utc::now().timestamp();
                        let group = job_clone.job.group();
                        let name = job_clone.job.name();
                        let span = tracing::error_span!("run_pending", group, name, timestamp);
                        let _enter = span.enter();

                        info!("Start execution of Job [{}/{}]", group, name);
                        match job_clone.run() {
                            Ok(()) => {
                                info!(
                                    "Execution of Job [{}/{}] completed successfully",
                                    group, name
                                );
                            }
                            Err(err) => {
                                error!(
                                    "Execution of Job [{}/{}] completed with errors. Err: {}",
                                    group, name, err
                                );
                            }
                        }
                    });
                } else {
                    debug!(
                        "Job [{}/{}] is pending but already running. It will not be executed.",
                        job_scheduler.job.group(),
                        job_scheduler.job.name()
                    )
                }
            }
        }
    }

    /// Adds a job to the JobExecutor.
    pub fn add_job(
        &mut self,
        schedule: &dyn TryToScheduler,
        job: Job,
    ) -> Result<(), SchedulerError> {
        self.add_job_with_scheduler(schedule.to_scheduler()?, job);
        Ok(())
    }

    /// Adds a job to the JobExecutor.
    pub fn add_job_with_multi_schedule(
        &mut self,
        schedule: &[&dyn TryToScheduler],
        job: Job,
    ) -> Result<(), SchedulerError> {
        self.add_job_with_scheduler(schedule.to_scheduler()?, job);
        Ok(())
    }

    /// Adds a job to the JobExecutor.
    pub fn add_job_with_scheduler<S: Into<Scheduler>>(&mut self, schedule: S, job: Job) {
        self.jobs.push(Arc::new(JobScheduler::new(
            schedule.into(),
            self.timezone,
            job,
        )));
    }

    pub fn set_sleep_between_checks(&mut self, sleep: Duration) {
        self.sleep_between_checks = sleep;
    }

    /// Starts the JobExecutor
    pub fn run(&self) {
        let mut running = self.is_running();
        if !running {
            info!("Starting the job executor");
            {
                let mut write = self.running.write();
                *write = true;
            };
            running = true;
            while running {
                self.run_pending_jobs();
                std::thread::sleep(self.sleep_between_checks);
                running = self.is_running();
            }
        } else {
            warn!("The JobExecutor is already running.")
        }
    }

    /// Stops the JobExecutor
    pub fn stop(&self, grateful: bool) {
        let running = self.is_running();
        if running {
            info!("Stopping the job executor");
            {
                let mut write = self.running.write();
                *write = false;
            };
            if grateful {
                info!("Wait for all Jobs to complete");
                while self.is_running_job() {
                    std::thread::sleep(self.sleep_between_checks);
                }
                info!("All Jobs completed");
            }
        } else {
            warn!("The JobExecutor is not running.")
        }
    }
}

impl Drop for JobExecutor {
    fn drop(&mut self) {
        self.stop(true);
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use chrono::Utc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn should_not_run_an_already_running_job() {
        let mut executor = new_executor_with_utc_tz();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        let (tx, rx) = channel();

        executor
            .add_job(
                &Duration::new(0, 1),
                Job::new("g", "n", None, move || {
                    tx.send("").unwrap();
                    println!("job - started");
                    count_clone.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::new(1, 0));
                    Ok(())
                }),
            )
            .unwrap();

        for i in 0..100 {
            println!("run_pending {}", i);
            executor.run_pending_jobs();
            std::thread::sleep(Duration::new(0, 2));
        }

        println!("run_pending completed");
        rx.recv().unwrap();

        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn a_running_job_should_not_block_the_executor() {
        let mut executor = new_executor_with_local_tz();

        let (tx, rx) = channel();

        let count_1 = Arc::new(AtomicUsize::new(0));
        let count_clone_1 = count_1.clone();
        let tx_1 = tx.clone();
        executor
            .add_job_with_multi_schedule(
                &[&Duration::new(0, 1)],
                Job::new("g", "n", None, move || {
                    tx_1.send("").unwrap();
                    println!("job 1 - started");
                    count_clone_1.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::new(1, 0));
                    Ok(())
                }),
            )
            .unwrap();

        let count_2 = Arc::new(AtomicUsize::new(0));
        let count_2_clone = count_2.clone();
        let tx_2 = tx.clone();
        executor
            .add_job(
                &Duration::new(0, 1),
                Job::new("g", "n", None, move || {
                    tx_2.send("").unwrap();
                    println!("job 2 - started");
                    count_2_clone.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::new(1, 0));
                    Ok(())
                }),
            )
            .unwrap();

        let count_3 = Arc::new(AtomicUsize::new(0));
        let count_3_clone = count_3.clone();
        let tx_3 = tx.clone();
        executor
            .add_job(
                &Duration::new(0, 1),
                Job::new("g", "n", None, move || {
                    tx_3.send("").unwrap();
                    println!("job 3 - started");
                    count_3_clone.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::new(1, 0));
                    Ok(())
                }),
            )
            .unwrap();

        let before_millis = Utc::now().timestamp_millis();
        for i in 0..100 {
            println!("run_pending {}", i);
            executor.run_pending_jobs();
            std::thread::sleep(Duration::new(0, 1_000_000));
        }
        let after_millis = Utc::now().timestamp_millis();

        assert!((after_millis - before_millis) >= 100);
        assert!((after_millis - before_millis) < 1000);

        rx.recv().unwrap();

        assert_eq!(count_1.load(Ordering::SeqCst), 1);
        assert_eq!(count_2.load(Ordering::SeqCst), 1);
        assert_eq!(count_3.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn should_gracefully_shutdown_the_job_executor() {
        let mut executor = new_executor_with_utc_tz();

        let count = Arc::new(AtomicUsize::new(0));

        let tasks = 100;

        for _i in  0..tasks {
            let count_clone = count.clone();
            executor
                .add_job(
                    &Duration::new(0, 1),
                    Job::new("g", "n", None, move || {
                        std::thread::sleep(Duration::new(1, 0));
                        println!("job - started");
                        count_clone.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    }),
                )
                .unwrap();
        };

        executor.set_sleep_between_checks(Duration::from_millis(10));

        let executor = Arc::new(executor);
        let executor_clone = executor.clone();
        std::thread::spawn(move || {
            executor_clone.run();
        });

        loop {
            if executor.is_running_job() {
                break;
            }
            std::thread::sleep(Duration::from_nanos(1));
        }

        executor.stop(true);

        assert_eq!(count.load(Ordering::Relaxed), tasks);
    }

    #[test]
    fn should_add_with_explicit_scheduler() {
        let mut executor = new_executor_with_utc_tz();
        executor.add_job_with_scheduler(Scheduler::Never, Job::new("g", "n", None, move || Ok(())));
    }

    #[test]
    fn should_register_a_schedule_by_vec() {
        let mut executor = new_executor_with_utc_tz();
        executor
            .add_job(
                &vec!["0 1 * * * * *"],
                Job::new("g", "n", None, move || Ok(())),
            )
            .unwrap();
        executor
            .add_job(
                &vec!["0 1 * * * * *".to_owned(), "0 1 * * * * *".to_owned()],
                Job::new("g", "n", None, move || Ok(())),
            )
            .unwrap();
    }
}
