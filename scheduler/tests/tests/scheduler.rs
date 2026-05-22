//! End-to-end `JobExecutor` integration tests.
//!
//! Generic over `crate::RepoUnderTest` so the same tests run against both
//! the Postgres backend (from `postgres_it.rs`) and the memory backend
//! (from `memory_it.rs`).

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use lightspeed_scheduler::{Job, JobExecutor, ScheduleRepository, ScheduledTask, Scheduler};
use lightspeed_test_utils::tokio_test;

use crate::utils::unique_name;
// `RepoUnderTest` is the concrete repository the running binary tests
// against; each `*_it.rs` binary defines its own alias.
use crate::*;

/// `Scheduler::Interval` configured to fire immediately on the first poll
/// and then every hour after — used by tests that just want "due now".
fn due_now() -> Scheduler {
    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true }
}

/// Records the number of times it ran and lets the caller inspect that count.
struct CountingTask {
    count: Arc<AtomicUsize>,
}

impl ScheduledTask<RepoUnderTest> for CountingTask {
    type Error = Infallible;
    async fn run(&self, _tx: &mut <RepoUnderTest as ScheduleRepository>::Tx) -> Result<(), Self::Error> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn executor_does_not_fire_when_not_due() {
    tokio_test(async {
        let d = data(false).await;
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(d.0.clone());
        executor
            .add_job(
                &Duration::from_secs(3600),
                Job::new(unique_name(), unique_name(), None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 0);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    });
}

#[test]
fn executor_fires_and_advances_when_due() {
    tokio_test(async {
        let d = data(false).await;
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(d.0.clone());
        executor
            .add_job_with_scheduler(
                due_now(),
                Job::new(unique_name(), unique_name(), None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // After firing once, next_run_at is +1h, so a second tick is a no-op.
        assert_eq!(executor.tick().await.unwrap(), 0);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn executor_fires_multiple_jobs_in_one_tick() {
    tokio_test(async {
        let d = data(false).await;
        let count_a = Arc::new(AtomicUsize::new(0));
        let count_b = Arc::new(AtomicUsize::new(0));
        let count_c = Arc::new(AtomicUsize::new(0));
        let group = unique_name();

        let executor = JobExecutor::new_with_utc_tz(d.0.clone());
        executor
            .add_job_with_scheduler(
                due_now(),
                Job::new(group.clone(), unique_name(), None, CountingTask { count: Arc::clone(&count_a) }),
            )
            .await
            .unwrap();
        executor
            .add_job_with_scheduler(
                due_now(),
                Job::new(group.clone(), unique_name(), None, CountingTask { count: Arc::clone(&count_b) }),
            )
            .await
            .unwrap();
        // Not due — should not fire.
        executor
            .add_job(
                &Duration::from_secs(3600),
                Job::new(group, unique_name(), None, CountingTask { count: Arc::clone(&count_c) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 2);
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
        assert_eq!(count_c.load(Ordering::SeqCst), 0);
    });
}

#[test]
fn two_executors_with_same_group_and_name_fire_only_once() {
    /// Holds the transaction long enough that two concurrent ticks actually
    /// contend on the row lock (otherwise the first one would commit before
    /// the second one even tried to claim).
    struct SlowTask {
        count: Arc<AtomicUsize>,
    }
    impl ScheduledTask<RepoUnderTest> for SlowTask {
        type Error = Infallible;
        async fn run(&self, _tx: &mut <RepoUnderTest as ScheduleRepository>::Tx) -> Result<(), Self::Error> {
            tokio::time::sleep(Duration::from_millis(200)).await;
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    tokio_test(async {
        let d = data(false).await;
        let group = unique_name();
        let name = unique_name();
        let count = Arc::new(AtomicUsize::new(0));

        // Register with `execute_at_startup: true` + 1h interval — the row
        // is due immediately (so the contending ticks below actually have a
        // due row to race for), but after the winner fires, `next_run_at`
        // advances 1h into the future and the loser's poll sees nothing.
        // Both executors register the same `(group, name)`; the second
        // register is a no-op against the persisted row (matching
        // fingerprint) but it still seeds executor_b's local
        // `next_run_at` mirror to "now", which is what `tick`'s local
        // due-check requires before it'll spawn a `tick_one`.
        let executor_a = JobExecutor::new_with_utc_tz(d.0.clone());
        executor_a
            .add_job_with_scheduler(
                due_now(),
                Job::new(group.clone(), name.clone(), None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();
        let executor_b = JobExecutor::new_with_utc_tz(d.0.clone());
        executor_b
            .add_job_with_scheduler(
                due_now(),
                Job::new(group.clone(), name.clone(), None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        // Run both ticks concurrently. The FOR UPDATE SKIP LOCKED lock means
        // exactly one of them should fire; the other should see no due row.
        let (a, b) = tokio::join!(executor_a.tick(), executor_b.tick());
        let fired_a = a.unwrap();
        let fired_b = b.unwrap();
        assert_eq!(fired_a + fired_b, 1, "exactly one executor must fire (a={fired_a}, b={fired_b})",);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn failing_job_keeps_schedule_due() {
    struct FailingTask;
    impl ScheduledTask<RepoUnderTest> for FailingTask {
        type Error = std::io::Error;
        async fn run(&self, _tx: &mut <RepoUnderTest as ScheduleRepository>::Tx) -> Result<(), Self::Error> {
            Err(std::io::Error::other("boom"))
        }
    }

    tokio_test(async {
        let d = data(false).await;
        let repo = d.0.clone();
        let group = unique_name();
        let name = unique_name();
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(due_now(), Job::new(group.clone(), name.clone(), None, FailingTask))
            .await
            .unwrap();

        // Task error is absorbed; tick reports 0 successful firings.
        assert_eq!(executor.tick().await.unwrap(), 0);

        // The advance was rolled back, so the schedule should still be due
        // and immediately claimable.
        let mut tx = repo.begin().await.unwrap();
        let row = repo.try_claim_due(&mut tx, &group, &name).await.unwrap();
        assert!(row.is_some(), "schedule must remain due after task failure");
        repo.commit(tx).await.unwrap();
    });
}
