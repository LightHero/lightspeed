use crate::schedule::Schedule;
use chrono::{DateTime, Utc};

pub struct Job {
    function: Box<FnMut() -> Result<(), Box<std::error::Error>> + Send + Sync>,
    group: String,
    name: String,
    schedule: Schedule,
    next_run_at: Option<DateTime<Utc>>,
    last_run_at: Option<DateTime<Utc>>,
    is_active: bool,
    is_running: bool,
}

impl Job {
    pub fn new<
        G: Into<String>,
        N: Into<String>,
        S: Into<Schedule>,
        F: FnMut() -> Result<(), Box<std::error::Error>> + Send + Sync,
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
            function: Box::new(function),
            name: name.into(),
            group: group.into(),
            schedule: schedule.into(),
            next_run_at: None,
            last_run_at: None,
            is_running: false,
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
        if let Some(next_run_at) = self.next_run_at {
            Utc::now() >= next_run_at
        } else {
            false
        }
    }

    /// Returns true if this job is currently running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    /// Run the job immediately and re-schedule it.
    pub fn run(&mut self) -> Result<(), Box<std::error::Error>> {
        // Execute the job function
        let run_result = (self.function)();

        // Save the last time this ran
        self.last_run_at = Some(Utc::now());

        // Determine the next time it should run
        self.next_run_at = self.schedule.next(self.last_run_at);

        run_result
    }
}
