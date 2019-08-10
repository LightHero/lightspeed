pub mod error;
pub mod job;
pub mod schedule;

use crate::job::Job;
use log::*;

#[derive(Default)]
pub struct Scheduler {
    jobs: Vec<Job>,
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
        for job in &mut self.jobs {
            if job.is_pending() {
                info!("Start execution of Job [{}/{}]", job.group(), job.name());
                match job.run() {
                    Ok(()) => {
                        info!(
                            "Execution of Job [{}/{}] completed successfully",
                            job.group(),
                            job.name()
                        );
                    }
                    Err(err) => {
                        error!(
                            "Execution of Job [{}/{}] completed with errors. Err: {}",
                            job.group(),
                            job.name(),
                            err
                        );
                    }
                }
            }
        }
    }

    /// Adds a job to the Scheduler.
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }
}
