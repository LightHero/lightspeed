use crate::error::SchedulerError;
use chrono::prelude::*;
use cron;
use std::convert::TryFrom;
use std::time::Duration;

pub enum Schedule {
    /// Set to execute on set time periods
    Periodic(cron::Schedule),

    /// Set to execute exactly `duration` away from the previous execution
    Interval(Duration),

    /// Set to execute to never
    Never,
}

impl Schedule {
    // Determine the next time we should execute (from a reference point)
    pub fn next(&self, after: &Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
        let after = after.unwrap_or_else(Utc::now);

        match *self {
            Schedule::Periodic(ref cs) => cs.after(&after).next(),

            Schedule::Interval(ref duration) => {
                let ch_duration = match time::Duration::from_std(*duration) {
                    Ok(value) => value,
                    Err(_) => {
                        return None;
                    }
                };

                Some(after + ch_duration)
            }

            Schedule::Never => None,
        }
    }
}

impl<'a> TryFrom<&'a str> for Schedule {
    type Error = SchedulerError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Schedule::Periodic(value.parse().map_err(|err| {
            SchedulerError::ScheduleDefinitionError {
                message: format!("Cannot create schedule for [{}]. Err: {}", value, err),
            }
        })?))
    }
}

impl TryFrom<Duration> for Schedule {
    type Error = SchedulerError;
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Schedule::Interval(value))
    }
}
