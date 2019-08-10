pub mod error;
pub mod job;
pub mod schedule;

use crate::job::Job;
use log::*;
use std::sync::Arc;

#[derive(Default)]
pub struct Scheduler {
    jobs: Vec<Arc<Job>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler::default()
    }

    /// Returns true if the Scheduler contains no jobs.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Returns the number of jobs in the Scheduler.
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Clear the Scheduler, removing all jobs.
    pub fn clear(&mut self) {
        self.jobs.clear()
    }

    /// Returns true if there is at least one job pending.
    pub fn is_pending(&self) -> bool {
        for job in &self.jobs {
            if job.is_pending() {
                return true;
            }
        }
        false
    }

    /// Run pending jobs in the Scheduler.
    pub fn run_pending(&mut self) {
        for job in &self.jobs {
            if job.is_pending() {
                match job.is_running() {
                    Ok(is_running) => {
                        if !is_running {
                            let job_clone = job.clone();
                            std::thread::spawn(move || {
                                info!("Start execution of Job [{}/{}]", job_clone.group(), job_clone.name());
                                match job_clone.run() {
                                    Ok(()) => {
                                        info!(
                                            "Execution of Job [{}/{}] completed successfully",
                                            job_clone.group(),
                                            job_clone.name()
                                        );
                                    }
                                    Err(err) => {
                                        error!(
                                            "Execution of Job [{}/{}] completed with errors. Err: {}",
                                            job_clone.group(),
                                            job_clone.name(),
                                            err
                                        );
                                    }
                                }
                            });
                        } else {
                            debug!("Job [{}/{}] is pending but already running. It will not be executed.", job.group(), job.name())
                        }
                    },
                    Err(err) => error!(
                        "Cannot start execution of Job [{}/{}] because status is unknown. Err: {}",
                        job.group(),
                        job.name(),
                        err
                    )
                }
            }
        };
    }

    /// Adds a job to the Scheduler.
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(Arc::new(job));
    }
}
