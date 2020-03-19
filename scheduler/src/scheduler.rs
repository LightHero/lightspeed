use crate::error::SchedulerError;
use chrono::prelude::*;
use chrono_tz::Tz;
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
    pub fn next(&self, after: &DateTime<Utc>, timezone: Option<Tz>) -> Option<DateTime<Utc>> {
        match *self {
            Scheduler::Cron(ref cs) => {
                if let Some(tz) = timezone {
                    cs.after(&after.with_timezone(&tz)).next().map(|date| date.with_timezone(&Utc))
                } else {
                    cs.after(&after).next()
                }
            }

            Scheduler::Interval(ref duration) => {
                let ch_duration = match time::Duration::from_std(*duration) {
                    Ok(value) => value,
                    Err(_) => {
                        return None;
                    }
                };
                Some(*after + ch_duration)
            }

            Scheduler::Never => None,
        }
    }
}

impl<'a> TryFrom<&'a str> for Scheduler {
    type Error = SchedulerError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Scheduler::Cron(value.parse().map_err(|err| SchedulerError::ScheduleDefinitionError {
            message: format!("Cannot create schedule for [{}]. Err: {}", value, err),
        })?))
    }
}

impl TryFrom<String> for Scheduler {
    type Error = SchedulerError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        use std::convert::TryInto;
        value.as_str().try_into()
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
    use chrono_tz::UTC;
    use std::convert::TryInto;

    #[test]
    fn never_should_not_schedule() {
        let schedule = Scheduler::Never;
        assert_eq!(None, schedule.next(&Utc::now(), Some(UTC)))
    }

    #[test]
    fn interval_should_schedule_plus_duration() {
        let now = Utc::now();
        let secs = 10;
        let schedule: Scheduler = Duration::new(secs, 0).try_into().unwrap();

        let next = schedule.next(&now, None).unwrap();

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

        let next = schedule.next(&date, Some(UTC)).unwrap();

        assert_eq!(next, expected_utc);
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_custom_time_zone() {
        let schedule: Scheduler = "* 11 10 * * *".try_into().unwrap();

        let date = Utc.ymd(2010, 1, 1).and_hms(10, 10, 0);
        let expected_utc = Utc.ymd(2010, 1, 2).and_hms(09, 11, 0);

        let tz = chrono_tz::Europe::Rome;

        let next = schedule.next(&date, Some(tz)).unwrap();

        assert_eq!(next.with_timezone(&Utc), expected_utc);
    }
}
