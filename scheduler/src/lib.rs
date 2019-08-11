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
            //println!("check JOB IS PENDING: {}", job.is_pending());
            if job.is_pending() {
                match job.is_running() {
                    Ok(is_running) => {
                        //println!("JOB IS RUNNING? {}", is_running);
                        if !is_running {
                            let job_clone = job.clone();
                            std::thread::spawn(move || {
                                info!(
                                    "Start execution of Job [{}/{}]",
                                    job_clone.group(),
                                    job_clone.name()
                                );
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
                    }
                    Err(err) => error!(
                        "Cannot start execution of Job [{}/{}] because status is unknown. Err: {}",
                        job.group(),
                        job.name(),
                        err
                    ),
                }
            }
        }
    }

    /// Adds a job to the Scheduler.
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(Arc::new(job));
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn should_not_run_an_already_running_job() {
        let mut scheduler = Scheduler::new();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        let (tx, rx) = channel();

        scheduler.add_job(
            Job::new("g", "n", Duration::new(0, 1), move || {
                tx.send("").unwrap();
                println!("job - started");
                count_clone.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::new(1, 0));
                Ok(())
            })
            .unwrap(),
        );

        for i in 0..100 {
            println!("run_pending {}", i);
            scheduler.run_pending();
            std::thread::sleep(Duration::new(0, 2));
        }

        println!("run_pending completed");
        rx.recv().unwrap();

        let job_executions = count.load(Ordering::Relaxed);
        assert_eq!(job_executions, 1);
    }

    #[test]
    fn a_runnig_job_should_not_block_the_scheduler() {
        assert!(false)
    }
}
