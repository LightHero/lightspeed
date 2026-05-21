use std::str::FromStr;
use std::time::{Duration, SystemTime};

use chrono::prelude::*;
use chrono_tz::Tz;

use crate::error::SchedulerError;

/// Schedule that can compute its next firing time.
///
/// Implementations are held by the [`JobExecutor`](crate::JobExecutor) and
/// are intentionally not persisted — only coordination state lives in the
/// repository.
///
/// The `&mut self` receiver mirrors the [`Scheduler`] enum's own
/// `Scheduler::next` method, which mutates state for variants like
/// [`Scheduler::Interval`] with `execute_at_startup`.
pub trait Schedule: Send + Sync + 'static {
    /// Next firing time strictly after `after`, in UTC. `None` means
    /// "never again". `timezone` is honoured by cron-style schedules.
    fn next(&mut self, after: &DateTime<Utc>, timezone: Option<Tz>) -> Option<DateTime<Utc>>;

    /// Stable identifier of this schedule's **definition** (not its
    /// instantaneous mutating state). Persisted next to the schedule row so
    /// a redeploy with a different schedule definition can detect the
    /// change and re-anchor `next_run_at`.
    ///
    /// Two schedules with the same observable firing behaviour should
    /// return the same string; two with different behaviour should differ.
    /// The default returns an empty string, which disables the
    /// change-detection optimisation (`register` will never re-anchor for
    /// schedules that don't override this method). Custom [`Schedule`]
    /// impls should override.
    fn fingerprint(&self) -> String {
        String::new()
    }
}

/// Built-in schedule kinds.
///
/// Variants mirror the old `lightspeed_scheduler` crate so that existing call
/// sites — including `Scheduler::Interval { interval_duration, execute_at_startup }`
/// struct literals — keep working.
pub enum Scheduler {
    /// Fire on every match of a cron expression.
    Cron(Box<cron::Schedule>),

    /// Fire exactly `interval_duration` after the previous firing. When
    /// `execute_at_startup` is `true`, the very first call to
    /// [`Schedule::next`] returns the reference time itself so the job runs
    /// immediately on startup.
    Interval { interval_duration: Duration, execute_at_startup: bool },

    /// Compose multiple schedules. Fires whenever any inner schedule fires
    /// (the earliest of all inner `next()` results wins).
    Multi(Vec<Scheduler>),

    /// Never fires.
    Never,
}

impl Scheduler {
    /// Builds a [`Scheduler`] from a slice of `&dyn TryToScheduler`. Empty
    /// slice becomes [`Scheduler::Never`]; a single element is unwrapped;
    /// multiple elements become [`Scheduler::Multi`].
    pub fn from(schedule: &[&dyn TryToScheduler]) -> Result<Scheduler, SchedulerError> {
        schedule.to_scheduler()
    }
}

impl Schedule for Scheduler {
    fn next(&mut self, after: &DateTime<Utc>, timezone: Option<Tz>) -> Option<DateTime<Utc>> {
        match *self {
            Scheduler::Cron(ref cs) => {
                if let Some(tz) = timezone {
                    cs.after(&after.with_timezone(&tz)).next().map(|date| date.with_timezone(&Utc))
                } else {
                    cs.after(after).next()
                }
            }
            Scheduler::Interval { ref interval_duration, ref mut execute_at_startup } => {
                if *execute_at_startup {
                    *execute_at_startup = false;
                    Some(*after)
                } else {
                    let ch_duration = chrono::Duration::from_std(*interval_duration).ok()?;
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

    fn fingerprint(&self) -> String {
        match self {
            // Debug of `cron::Schedule` round-trips the parsed expression
            // deterministically — same expression yields the same string.
            Scheduler::Cron(s) => format!("cron:{s:?}"),
            Scheduler::Interval { interval_duration, execute_at_startup } => format!(
                "interval:{}.{:09}:{}",
                interval_duration.as_secs(),
                interval_duration.subsec_nanos(),
                execute_at_startup,
            ),
            Scheduler::Multi(parts) => {
                let mut out = String::from("multi:[");
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&p.fingerprint());
                }
                out.push(']');
                out
            }
            Scheduler::Never => "never".to_string(),
        }
    }
}

/// Converts an ad-hoc input (e.g. a cron string or a [`Duration`]) into a
/// [`Scheduler`].
pub trait TryToScheduler {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError>;
}

impl TryToScheduler for Vec<String> {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        let refs: Vec<&str> = self.iter().map(|s| s.as_ref()).collect();
        refs.to_scheduler()
    }
}

impl TryToScheduler for Vec<&str> {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        match self.len() {
            0 => Ok(Scheduler::Never),
            1 => self[0].to_scheduler(),
            _ => {
                let mut result = vec![];
                for scheduler in self {
                    result.push(scheduler.to_scheduler()?);
                }
                Ok(Scheduler::Multi(result))
            }
        }
    }
}

impl TryToScheduler for Vec<&dyn TryToScheduler> {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        (&self[..]).to_scheduler()
    }
}

impl TryToScheduler for &[&dyn TryToScheduler] {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        match self.len() {
            0 => Ok(Scheduler::Never),
            1 => self[0].to_scheduler(),
            _ => {
                let mut result = vec![];
                for scheduler in *self {
                    result.push(scheduler.to_scheduler()?);
                }
                Ok(Scheduler::Multi(result))
            }
        }
    }
}

impl TryToScheduler for &str {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Cron(Box::new(cron::Schedule::from_str(self).map_err(|err| {
            SchedulerError::ScheduleDefinitionError {
                message: format!("Cannot create schedule for [{self}]. Err: {err:?}"),
            }
        })?)))
    }
}

impl TryToScheduler for String {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        self.as_str().to_scheduler()
    }
}

impl TryToScheduler for Duration {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Interval { interval_duration: *self, execute_at_startup: false })
    }
}

impl TryToScheduler for (Duration, bool) {
    fn to_scheduler(&self) -> Result<Scheduler, SchedulerError> {
        Ok(Scheduler::Interval { interval_duration: self.0, execute_at_startup: self.1 })
    }
}

impl From<Vec<Scheduler>> for Scheduler {
    fn from(val: Vec<Scheduler>) -> Self {
        Scheduler::Multi(val)
    }
}

/// Converts a `DateTime<Utc>` back to [`SystemTime`] for repository writes.
pub(crate) fn utc_to_system_time(t: DateTime<Utc>) -> SystemTime {
    SystemTime::from(t)
}

#[cfg(test)]
mod tests {
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
        let mut schedule = Duration::new(secs, 0).to_scheduler().unwrap();

        let next = schedule.next(&now, None).unwrap();

        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn interval_should_schedule_at_startup() {
        let now = Utc::now();
        let secs = 10;
        let mut schedule = (Duration::new(secs, 0), true).to_scheduler().unwrap();

        let first = schedule.next(&now, None).unwrap();
        assert_eq!(now.timestamp(), first.timestamp());

        let next = schedule.next(&now, None).unwrap();
        assert!(next.timestamp() >= now.timestamp() + (secs as i64));
    }

    #[test]
    fn should_build_an_interval_schedule_from_duration() {
        let schedule = Duration::new(1, 1).to_scheduler().unwrap();
        match schedule {
            Scheduler::Interval { .. } => (),
            _ => panic!(),
        }
    }

    #[test]
    fn should_build_a_periodic_schedule_from_str() {
        let schedule = "* * * * * *".to_scheduler().unwrap();
        match schedule {
            Scheduler::Cron(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_empty_array() {
        let schedule = Scheduler::from(&[]).unwrap();
        match schedule {
            Scheduler::Never => (),
            _ => panic!(),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_single_entry_array() {
        let schedule = Scheduler::from(&[&vec!["* * * * * *"]]).unwrap();
        match schedule {
            Scheduler::Cron(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn should_build_a_multi_scheduler_from_array() {
        let schedule = Scheduler::from(&[&"* * * * * *", &Duration::from_secs(9)]).unwrap();
        match schedule {
            Scheduler::Multi(inner) => {
                match inner[0] {
                    Scheduler::Cron(_) => (),
                    _ => panic!(),
                };
                match inner[1] {
                    Scheduler::Interval { .. } => (),
                    _ => panic!(),
                };
            }
            _ => panic!(),
        }
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_utc() {
        let mut schedule = "* 11 10 * * *".to_scheduler().unwrap();
        let date = Utc.with_ymd_and_hms(2010, 1, 1, 10, 10, 0).unwrap();

        let expected_utc = Utc.with_ymd_and_hms(2010, 1, 1, 10, 11, 0).unwrap();

        let next = schedule.next(&date, Some(UTC)).unwrap();

        assert_eq!(next, expected_utc);
    }

    #[test]
    fn cron_should_be_time_zone_aware_with_custom_time_zone() {
        let mut schedule = "* 11 10 * * *".to_scheduler().unwrap();

        let date = Utc.with_ymd_and_hms(2010, 1, 1, 10, 10, 0).unwrap();
        let expected_utc = Utc.with_ymd_and_hms(2010, 1, 2, 9, 11, 0).unwrap();

        let tz = chrono_tz::Europe::Rome;

        let next = schedule.next(&date, Some(tz)).unwrap();

        assert_eq!(next.with_timezone(&Utc), expected_utc);
    }

    #[test]
    fn multi_should_return_first_possible_next_execution() {
        let mut schedule = Scheduler::from(&[&"* 10 10 * * *", &"* 20 20 * * *"]).unwrap();

        {
            let date = Utc.with_ymd_and_hms(2010, 1, 1, 10, 8, 0).unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 1, 1, 10, 10, 0).unwrap();

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }

        {
            let date = Utc.with_ymd_and_hms(2010, 1, 1, 11, 8, 0).unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 1, 1, 20, 20, 0).unwrap();

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }

        {
            let date = Utc.with_ymd_and_hms(2010, 1, 1, 22, 8, 0).unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 1, 2, 10, 10, 0).unwrap();

            let next = schedule.next(&date, Some(UTC)).unwrap();
            assert_eq!(next.with_timezone(&Utc), expected);
        }
    }
}
