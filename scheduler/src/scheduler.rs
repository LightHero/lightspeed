use crate::error::SchedulerError;
use chrono::prelude::*;
use chrono_tz::Tz;
use std::time::Duration;

pub enum Scheduler {
    /// Set to execute on set time periods
    Cron(cron::Schedule),

    /// Set to execute exactly `duration` away from the previous execution.
    /// If
    Interval {
        interval_duration: Duration,
        execute_at_startup: bool,
    },

    /// Multi shceduler: the execution is trigger where at least one of the schedulers in matched
    Multi(Vec<Scheduler>),

    /// Set to execute to never
    Never,
}

impl Scheduler {
    pub fn from(schedule: &[&dyn IntoScheduler]) -> Result<Scheduler, SchedulerError> {
        schedule.into_scheduler()
    }

    // Determine the next time we should execute (from a reference point)
    pub fn next(&mut self, after: &DateTime<Utc>, timezone: Option<Tz>) -> Option<DateTime<Utc>> {
        match *self {
            Scheduler::Cron(ref cs) => {
                if let Some(tz) = timezone {
                    cs.after(&after.with_timezone(&tz))
                        .next()
                        .map(|date| date.with_timezone(&Utc))
                } else {
                    cs.after(&after).next()
                }
            }

            Scheduler::Interval {
                ref interval_duration,
                ref mut execute_at_startup,
            } => {
                if *execute_at_startup {
                    *execute_at_startup = false;
                    Some(*after)
                } else {
                    let ch_duration = match time::Duration::from_std(*interval_duration) {
                        Ok(value) => value,
                        Err(_) => {
                            return None;
                        }
                    };
                    Some(*after + ch_duration)
                }
            }

            Scheduler::Multi(ref mut schedulers) => {
                let mut result = None;
                for scheduler in schedulers {
                    if let Some(local_next) = scheduler.next(after, timezone) {
                        result = match result {
                            Some(current_next) => {
                                if local_next < current_next {
                                    Some(local_next)
                                } else {
                                    Some(current_next)
                                }
                            }
                            None => Some(local_next),
                        }
                    }
                }
                result
            }

            Scheduler::Never => None,
        }
    }
}

pub trait IntoScheduler {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError>;
}

impl IntoScheduler for Vec<&str> {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        match self.len() {
            0 => Ok(Scheduler::Never),
            1 => self[0].into_scheduler(),
            _ => {
                let mut result = vec![];
                for scheduler in self {
                    result.push(scheduler.into_scheduler()?);
                }
                Ok(Scheduler::Multi(result))
            }
        }
    }
}

impl IntoScheduler for Vec<&dyn IntoScheduler> {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        (&self[..]).into_scheduler()
    }
}

impl IntoScheduler for &[&dyn IntoScheduler] {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        match self.len() {
            0 => Ok(Scheduler::Never),
            1 => self[0].into_scheduler(),
            _ => {
                let mut result = vec![];
                for scheduler in *self {
                    result.push(scheduler.into_scheduler()?);
                }
                Ok(Scheduler::Multi(result))
            }
        }
    }
}

impl<'a> IntoScheduler for &'a str {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Cron(self.parse().map_err(|err| {
            SchedulerError::ScheduleDefinitionError {
                message: format!("Cannot create schedule for [{}]. Err: {}", self, err),
            }
        })?))
    }
}

impl IntoScheduler for String {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        self.as_str().into_scheduler()
    }
}

impl IntoScheduler for Duration {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Interval {
            interval_duration: *self,
            execute_at_startup: false,
        })
    }
}

impl IntoScheduler for (Duration, bool) {
    fn into_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Interval {
            interval_duration: self.0,
            execute_at_startup: self.1,
        })
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use chrono_tz::UTC;

    #[test]
    fn never_should_not_schedule() {
        let mut schedule = Scheduler::Never;
        assert_eq!(None, schedule.next(&Utc::now(), Some(UTC)))
    }

    #[test]
    fn interval_should_schedule_plus_duration() {
        let now = Utc::now();
        let secs = 10;
        let mut schedule = Duration::new(secs, 0).into_scheduler().unwrap();

        let next = schedule.next(&now, None).unwrap();

        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn interval_should_schedule_at_startup() {
        let now = Utc::now();
        let secs = 10;
        let mut schedule = (Duration::new(secs, 0), true).into_scheduler().unwrap();

        let first = schedule.next(&now, None).unwrap();
        assert_eq!(now.timestamp(), first.timestamp());

        let next = schedule.next(&now, None).unwrap();
        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn should_build_an_interval_schedule_from_duration() {
        let schedule = Duration::new(1, 1).into_scheduler().unwrap();
        match schedule {
            Scheduler::Interval { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_periodic_schedule_from_str() {
        let schedule = "* * * * * *".into_scheduler().unwrap();
        match schedule {
            Scheduler::Cron(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_empty_array() {
        let schedule = Scheduler::from(&vec![]).unwrap();
        match schedule {
            Scheduler::Never => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_single_entry_array() {
        let schedule = Scheduler::from(&[&vec!["* * * * * *"]]).unwrap();
        match schedule {
            Scheduler::Cron(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_array() {
        let schedule = Scheduler::from(&[&"* * * * * *", &Duration::from_secs(9)]).unwrap();
        match schedule {
            Scheduler::Multi(inner) => {
                match inner[0] {
                    Scheduler::Cron(_) => assert!(true),
                    _ => assert!(false),
                };
                match inner[1] {
                    Scheduler::Interval { .. } => assert!(true),
                    _ => assert!(false),
                };
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_utc() {
        let mut schedule = "* 11 10 * * *".into_scheduler().unwrap();
        let date = Utc.ymd(2010, 1, 1).and_hms(10, 10, 0);

        let expected_utc = Utc.ymd(2010, 1, 1).and_hms(10, 11, 0);

        let next = schedule.next(&date, Some(UTC)).unwrap();

        assert_eq!(next, expected_utc);
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_custom_time_zone() {
        let mut schedule = "* 11 10 * * *".into_scheduler().unwrap();

        let date = Utc.ymd(2010, 1, 1).and_hms(10, 10, 0);
        let expected_utc = Utc.ymd(2010, 1, 2).and_hms(09, 11, 0);

        let tz = chrono_tz::Europe::Rome;

        let next = schedule.next(&date, Some(tz)).unwrap();

        assert_eq!(next.with_timezone(&Utc), expected_utc);
    }

    #[test]
    fn multi_should_return_first_possible_next_execution() {
        let mut schedule = Scheduler::from(&[&"* 10 10 * * *", &"* 20 20 * * *"]).unwrap();

        {
            let date = Utc.ymd(2010, 1, 1).and_hms(10, 8, 0);
            let expected = Utc.ymd(2010, 1, 1).and_hms(10, 10, 0);

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }

        {
            let date = Utc.ymd(2010, 1, 1).and_hms(11, 8, 0);
            let expected = Utc.ymd(2010, 1, 1).and_hms(20, 20, 0);

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }

        {
            let date = Utc.ymd(2010, 1, 1).and_hms(22, 8, 0);
            let expected = Utc.ymd(2010, 1, 2).and_hms(10, 10, 0);

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }

    }

}
