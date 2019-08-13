use crate::error::SchedulerError;
use chrono::prelude::*;
use cron;
use std::convert::TryFrom;
use std::time::Duration;

pub enum Scheduler {
    /// Set to execute on set time periods
    Cron(cron::Schedule),

    /// Set to execute exactly `duration` away from the previous execution
    Interval(Duration),

    /// Set to execute to never
    Never,
}

impl Scheduler {
    // Determine the next time we should execute (from a reference point)
    pub fn next<T: TimeZone>(&self, after: &DateTime<T>) -> Option<DateTime<T>> {
        match *self {
            Scheduler::Cron(ref cs) => cs.after(&after).next(),

            Scheduler::Interval(ref duration) => {
                let ch_duration = match time::Duration::from_std(*duration) {
                    Ok(value) => value,
                    Err(_) => {
                        return None;
                    }
                };

                let date = after.with_timezone(&Utc) + ch_duration;
                Some(date.with_timezone(&after.timezone()))
            }

            Scheduler::Never => None,
        }
    }
}

impl<'a> TryFrom<&'a str> for Scheduler {
    type Error = SchedulerError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Scheduler::Cron(value.parse().map_err(|err| {
            SchedulerError::ScheduleDefinitionError {
                message: format!("Cannot create schedule for [{}]. Err: {}", value, err),
            }
        })?))
    }
}

impl TryFrom<Duration> for Scheduler {
    type Error = SchedulerError;
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Scheduler::Interval(value))
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn never_should_not_schedule() {
        let schedule = Scheduler::Never;
        assert_eq!(None, schedule.next(&Utc::now()))
    }

    #[test]
    fn interval_should_schedule_plus_duration() {
        let now = Utc::now();
        let secs = 10;
        let schedule: Scheduler = Duration::new(secs, 0).try_into().unwrap();

        let next = schedule.next(&now).unwrap();

        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn should_build_an_interval_schedule_from_duration() {
        let schedule: Scheduler = Duration::new(1, 1).try_into().unwrap();
        match schedule {
            Scheduler::Interval(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_periodic_schedule_from_str() {
        let schedule: Scheduler = "* * * * * *".try_into().unwrap();
        match schedule {
            Scheduler::Cron(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_utc() {
        let schedule: Scheduler = "* 11 10 * * *".try_into().unwrap();
        let date = Utc.ymd(2010, 1, 1).and_hms(10, 10, 0);

        let expected_utc = Utc.ymd(2010, 1, 1).and_hms(10, 11, 0);
        let next = schedule.next(&date).unwrap();

        assert_eq!(next, expected_utc);
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_custom_time_zone() {
        let schedule: Scheduler = "* 11 10 * * *".try_into().unwrap();

        let date = FixedOffset::east(3600).ymd(2010, 1, 1).and_hms(10, 10, 0);

        let expected_utc = Utc.ymd(2010, 1, 1).and_hms(9, 11, 0);

        let next = schedule.next(&date).unwrap();

        assert_eq!(next.with_timezone(&Utc), expected_utc);
    }

}
