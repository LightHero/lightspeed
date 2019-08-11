use crate::error::SchedulerError;
use chrono::prelude::*;
use cron;
use std::convert::TryFrom;
use std::time::Duration;

pub enum Schedule {
    /// Set to execute on set time periods
    Cron(cron::Schedule),

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
            Schedule::Cron(ref cs) => cs.after(&after).next(),

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
        Ok(Schedule::Cron(value.parse().map_err(|err| {
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

#[cfg(test)]
pub mod test {

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn never_should_not_schedule() {
        let schedule = Schedule::Never;
        assert_eq!(None, schedule.next(&None))
    }

    #[test]
    fn interval_should_schedule_plus_duration() {
        let now = Utc::now();
        let secs = 10;
        let schedule: Schedule = Duration::new(secs, 0).try_into().unwrap();

        let next = schedule.next(&Some(now)).unwrap();

        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn should_build_an_interval_schedule_from_duration() {
        let schedule: Schedule = Duration::new(1, 1).try_into().unwrap();
        match schedule {
            Schedule::Interval(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_periodic_schedule_from_str() {
        let schedule: Schedule = "* * * * * *".try_into().unwrap();
        match schedule {
            Schedule::Cron(_) => assert!(true),
            _ => assert!(false),
        }
    }

}
