#![doc = include_str!("../README.md")]

pub mod error;
pub mod job;
pub mod repository;
pub mod scheduler;

pub use error::SchedulerError;
pub use job::{FnTask, Job, ScheduledTask, TxBoxFuture};
pub use repository::*;
pub use scheduler::{Schedule, Scheduler, TryToScheduler};

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Utc;
use chrono_tz::{Tz, UTC};
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::task::{AbortHandle, JoinHandle};
use log::{debug, info, warn};

use crate::scheduler::utc_to_system_time;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type-erased view of a [`ScheduledTask`] so tasks with different `Error`
/// types can live in the same `Vec`.
trait ErasedTask<R: ScheduleRepository>: Send + Sync + 'static {
    fn run<'a>(&'a self, tx: &'a mut R::Tx) -> BoxedFuture<'a, Result<(), BoxedError>>;
}

struct TaskAdapter<T>(T);

impl<R, T> ErasedTask<R> for TaskAdapter<T>
where
    R: ScheduleRepository,
    T: ScheduledTask<R>,
{
    fn run<'a>(&'a self, tx: &'a mut R::Tx) -> BoxedFuture<'a, Result<(), BoxedError>> {
        Box::pin(async move { self.0.run(tx).await.map_err(|e| Box::new(e) as BoxedError) })
    }
}

struct Entry<R: ScheduleRepository> {
    group: String,
    name: String,
    schedule: Arc<StdMutex<Box<dyn Schedule>>>,
    task: Arc<dyn ErasedTask<R>>,
    retries_after_failure: Option<u64>,
}

impl<R: ScheduleRepository> Clone for Entry<R> {
    fn clone(&self) -> Self {
        Self {
            group: self.group.clone(),
            name: self.name.clone(),
            schedule: Arc::clone(&self.schedule),
            task: Arc::clone(&self.task),
            retries_after_failure: self.retries_after_failure,
        }
    }
}

struct RunningState {
    stop_request: Arc<Notify>,
    finished_rx: oneshot::Receiver<()>,
    abort_handle: AbortHandle,
}

/// Distributed executor for one or more named jobs.
///
/// Register jobs with [`add_job`](Self::add_job) (or
/// [`add_job_with_scheduler`](Self::add_job_with_scheduler) /
/// [`add_job_with_multi_schedule`](Self::add_job_with_multi_schedule)), then
/// call [`run`](Self::run) to spawn the polling loop as a tokio task; `run`
/// returns a `JoinHandle` you can `await` or `abort`. Stop the loop with
/// [`stop`](Self::stop).
///
/// Multiple processes running the same set of jobs against a shared
/// [`ScheduleRepository`] cooperate via row-level locking — exactly one
/// process fires each scheduled tick of a given job.
pub struct JobExecutor<R: ScheduleRepository> {
    repo: R,
    timezone: Option<Tz>,
    jobs: StdMutex<Vec<Entry<R>>>,
    running: AtomicBool,
    state: StdMutex<Option<RunningState>>,
    /// Signalled by `add_job` so a sleeping run loop wakes up promptly when a
    /// new job might be due sooner than its current sleep target.
    wake: Notify,
}

/// Upper bound on a single sleep interval — caps the wait when no schedule
/// has an imminent firing (or no jobs are registered yet). The `wake`
/// notification will usually break the sleep early; this is just a safety
/// net so the loop reconsiders state at least this often.
const MAX_SLEEP: Duration = Duration::from_secs(3600);

/// Far-future sentinel (year 9999) used for [`Scheduler::Never`] schedules
/// whose `next()` returns `None` — the row is registered but will never
/// become due.
fn never_sentinel() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(253_402_300_799)
}

impl<R: ScheduleRepository> JobExecutor<R> {
    /// Creates a new executor that uses the Local time zone for execution
    /// time evaluation. Cron expressions are interpreted against Local time.
    pub fn new_with_local_tz(repo: R) -> Self {
        Self::new_with_tz(repo, None)
    }

    /// Creates a new executor that uses the UTC time zone for execution time
    /// evaluation.
    pub fn new_with_utc_tz(repo: R) -> Self {
        Self::new_with_tz(repo, Some(UTC))
    }

    /// Creates a new executor with an explicit time zone.
    pub fn new_with_tz(repo: R, timezone: Option<Tz>) -> Self {
        Self {
            repo,
            timezone,
            jobs: StdMutex::new(Vec::new()),
            running: AtomicBool::new(false),
            state: StdMutex::new(None),
            wake: Notify::new(),
        }
    }

    /// Adds a job using any [`TryToScheduler`]-convertible schedule (cron
    /// string, `Duration`, etc.).
    pub async fn add_job<T>(
        &self,
        schedule: &dyn TryToScheduler,
        job: Job<T>,
    ) -> Result<(), SchedulerError>
    where
        T: ScheduledTask<R>,
    {
        let scheduler = schedule.to_scheduler()?;
        self.add_job_with_scheduler(scheduler, job).await
    }

    /// Adds a job whose schedule fires whenever any of the given expressions
    /// fires.
    pub async fn add_job_with_multi_schedule<T>(
        &self,
        schedule: &[&dyn TryToScheduler],
        job: Job<T>,
    ) -> Result<(), SchedulerError>
    where
        T: ScheduledTask<R>,
    {
        let scheduler = schedule.to_scheduler()?;
        self.add_job_with_scheduler(scheduler, job).await
    }

    /// Adds a job with an explicit [`Scheduler`] (or anything `Into<Scheduler>`).
    pub async fn add_job_with_scheduler<S, T>(
        &self,
        schedule: S,
        job: Job<T>,
    ) -> Result<(), SchedulerError>
    where
        S: Into<Scheduler>,
        T: ScheduledTask<R>,
    {
        let mut scheduler: Scheduler = schedule.into();
        let Job { group, name, retries_after_failure, task } = job;

        // Capture the fingerprint BEFORE `next()` mutates internal state
        // (specifically, `Interval { execute_at_startup }` flips to false on
        // first call). This way the fingerprint reflects the as-constructed
        // schedule and stays stable across restarts.
        let fingerprint = scheduler.fingerprint();

        let now_utc = Utc::now();
        let next_system = match scheduler.next(&now_utc, self.timezone) {
            Some(next_utc) => utc_to_system_time(next_utc),
            // Never-firing schedule (`Scheduler::Never`): register with a
            // far-future sentinel so the row is persisted but never claimed.
            None => never_sentinel(),
        };
        self.repo.register(&group, &name, next_system, &fingerprint).await?;

        info!(target: "lightspeed_scheduler", "add job to executor. group: {group}, name: {name}");
        let boxed: Box<dyn Schedule> = Box::new(scheduler);
        self.jobs.lock().unwrap().push(Entry {
            group,
            name,
            schedule: Arc::new(StdMutex::new(boxed)),
            task: Arc::new(TaskAdapter(task)),
            retries_after_failure,
        });
        self.wake.notify_one();
        Ok(())
    }

    /// Time to sleep before the next iteration, derived from the persisted
    /// `next_run_at` of every registered job. Capped at [`MAX_SLEEP`].
    async fn next_sleep_duration(&self) -> Duration {
        let owned: Vec<(String, String)> = self
            .jobs
            .lock()
            .unwrap()
            .iter()
            .map(|e| (e.group.clone(), e.name.clone()))
            .collect();
        let refs: Vec<(&str, &str)> = owned.iter().map(|(g, n)| (g.as_str(), n.as_str())).collect();
        let until = match self.repo.time_until_next_due(&refs).await {
            Ok(d) => d,
            Err(e) => {
                warn!(
                    target: "lightspeed_scheduler",
                    "failed to read next due time; falling back to MAX_SLEEP. error: {e}",
                );
                None
            }
        };
        until.unwrap_or(MAX_SLEEP).min(MAX_SLEEP)
    }

    /// Walks every registered job once, firing those that are due in
    /// parallel. Each job runs in its own spawned tokio task so a slow job
    /// can't block other due jobs from firing within the same tick cycle.
    /// Returns the number of jobs that fired successfully this cycle.
    pub async fn tick(&self) -> Result<usize, SchedulerError> {
        let snapshot = self.jobs.lock().unwrap().clone();
        let mut handles = Vec::with_capacity(snapshot.len());
        for entry in snapshot {
            let repo = self.repo.clone();
            let tz = self.timezone;
            handles.push(tokio::spawn(async move {
                Self::tick_one(repo, tz, entry).await
            }));
        }
        let mut fired = 0;
        for h in handles {
            match h.await {
                Ok(Ok(true)) => fired += 1,
                Ok(Ok(false)) => {}
                Ok(Err(e)) => return Err(e),
                Err(join_err) => {
                    warn!(
                        target: "lightspeed_scheduler",
                        "spawned tick task failed: {join_err}",
                    );
                }
            }
        }
        Ok(fired)
    }

    async fn tick_one(
        repo: R,
        tz: Option<Tz>,
        entry: Entry<R>,
    ) -> Result<bool, SchedulerError> {
        let mut tx = repo.begin().await?;

        let claimed = repo.try_claim_due(&mut tx, &entry.group, &entry.name).await?;
        if claimed.is_none() {
            repo.commit(tx).await?;
            return Ok(false);
        }

        // Retry policy: initial attempt + `retries_after_failure` retries.
        let max_attempts = entry.retries_after_failure.unwrap_or(0).saturating_add(1);
        let mut last_err: Option<BoxedError> = None;
        for attempt in 1..=max_attempts {
            match entry.task.run(&mut tx).await {
                Ok(()) => {
                    last_err = None;
                    break;
                }
                Err(e) => {
                    if attempt < max_attempts {
                        warn!(
                            target: "lightspeed_scheduler",
                            "task failed; retrying. group: {}, name: {}, attempt: {attempt}/{max_attempts}, error: {e}",
                            entry.group, entry.name,
                        );
                    }
                    last_err = Some(e);
                }
            }
        }

        if let Some(e) = last_err {
            let _ = repo.rollback(tx).await;
            warn!(
                target: "lightspeed_scheduler",
                "task failed after all retries. group: {}, name: {}, error: {e}",
                entry.group, entry.name,
            );
            return Ok(false);
        }

        let now_utc = Utc::now();
        let next_system = {
            let mut guard = entry.schedule.lock().unwrap();
            match guard.next(&now_utc, tz) {
                Some(next_utc) => utc_to_system_time(next_utc),
                None => never_sentinel(),
            }
        };
        let now_system = utc_to_system_time(now_utc);
        repo.advance(&mut tx, &entry.group, &entry.name, next_system, now_system).await?;
        repo.commit(tx).await?;
        debug!(
            target: "lightspeed_scheduler",
            "schedule fired. group: {}, name: {}",
            entry.group, entry.name,
        );
        Ok(true)
    }

    /// Spawns a tokio task that drives the executor and returns its
    /// `JoinHandle`. The loop computes the time until the earliest next
    /// firing across all registered schedules and sleeps until then.
    /// `add_job` wakes the loop early so newly-added jobs fire on time.
    ///
    /// Only one `run` can be active at a time per executor; concurrent calls
    /// return [`SchedulerError::JobExecutionStateError`]. The state is
    /// released when the loop exits.
    pub fn run(self: &Arc<Self>) -> Result<JoinHandle<()>, SchedulerError> {
        let mut state_guard = self.state.lock().unwrap();
        if self.running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(SchedulerError::JobExecutionStateError {
                message: "The JobExecutor is already running.".to_owned(),
            });
        }

        let stop_request = Arc::new(Notify::new());
        let (finished_tx, finished_rx) = oneshot::channel();
        let me_stop = Arc::clone(&stop_request);

        // Construct the cleanup OUTSIDE the async block and move it in.
        // Captured outer values are part of the future's initial state and
        // drop even if the task is aborted before its first poll.
        let cleanup = RunCleanup { me: Arc::clone(self), finished_tx: Some(finished_tx) };

        let me = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let _cleanup = cleanup;

            let job_count = me.jobs.lock().unwrap().len();
            info!(
                target: "lightspeed_scheduler",
                "JobExecutor started. jobs: {job_count}",
            );
            loop {
                if let Err(e) = me.tick().await {
                    warn!(target: "lightspeed_scheduler", "tick failed. error: {e}");
                }

                let sleep_for = me.next_sleep_duration().await;
                tokio::select! {
                    _ = me_stop.notified() => break,
                    // `add_job` rang the bell — re-tick and recompute the sleep.
                    _ = me.wake.notified() => {}
                    _ = tokio::time::sleep(sleep_for) => {}
                }
            }
            info!(target: "lightspeed_scheduler", "JobExecutor stopped");
        });

        *state_guard = Some(RunningState {
            stop_request,
            finished_rx,
            abort_handle: handle.abort_handle(),
        });
        drop(state_guard);

        Ok(handle)
    }

    /// Stops the running loop and waits for it to exit. Returns
    /// [`SchedulerError::JobExecutionStateError`] when no `run` is active.
    ///
    /// - `graceful = true`: signals the loop to break at its next select!
    ///   boundary, so the in-flight tick (if any) runs to completion.
    /// - `graceful = false`: aborts the spawned task immediately. The
    ///   in-flight tick is cancelled at its next await point; any open
    ///   transaction is rolled back when the connection is returned to the
    ///   pool.
    pub async fn stop(&self, graceful: bool) -> Result<(), SchedulerError> {
        let state = self.state.lock().unwrap().take().ok_or_else(|| {
            SchedulerError::JobExecutionStateError {
                message: "The JobExecutor is not running.".to_owned(),
            }
        })?;

        if graceful {
            state.stop_request.notify_one();
        } else {
            state.abort_handle.abort();
        }

        // `recv` returns Err if the sender was dropped without sending —
        // this happens on abort. Either way, the loop is gone.
        let _ = state.finished_rx.await;
        Ok(())
    }
}

/// Single drop guard captured into the spawned future. Clears the per-run
/// state, the `running` flag, and finally signals the finished channel so
/// `stop` callers wake up. Runs on normal exit, on `stop`'s abort, and even
/// if the task is dropped before its first poll.
struct RunCleanup<R: ScheduleRepository> {
    me: Arc<JobExecutor<R>>,
    finished_tx: Option<oneshot::Sender<()>>,
}

impl<R: ScheduleRepository> Drop for RunCleanup<R> {
    fn drop(&mut self) {
        *self.me.state.lock().unwrap() = None;
        self.me.running.store(false, Ordering::SeqCst);
        if let Some(tx) = self.finished_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::repository::MemoryScheduleRepository;

    struct CountingTask {
        count: Arc<AtomicUsize>,
    }

    impl ScheduledTask<MemoryScheduleRepository> for CountingTask {
        type Error = Infallible;
        async fn run(
            &self,
            _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
        ) -> Result<(), Self::Error> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    const G: &str = "test";

    #[tokio::test]
    async fn tick_returns_zero_when_no_job_due() {
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &Duration::from_secs(3600),
                Job::new(G, "task-1", None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 0);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn tick_fires_one_when_one_job_due() {
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "task-1", None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn tick_fires_multiple_jobs_in_one_cycle() {
        let count_a = Arc::new(AtomicUsize::new(0));
        let count_b = Arc::new(AtomicUsize::new(0));
        let count_c = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "a", None, CountingTask { count: Arc::clone(&count_a) }),
            )
            .await
            .unwrap();
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "b", None, CountingTask { count: Arc::clone(&count_b) }),
            )
            .await
            .unwrap();
        executor
            .add_job(
                &Duration::from_secs(3600),
                Job::new(G, "c", None, CountingTask { count: Arc::clone(&count_c) }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 2);
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
        assert_eq!(count_c.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn run_returns_already_running_when_called_twice() {
        let executor = Arc::new(JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init()));
        let handle = executor.run().unwrap();
        assert!(executor.running.load(Ordering::SeqCst));

        assert!(executor.run().is_err());

        executor.stop(false).await.unwrap();
        let _ = handle.await;
        assert!(!executor.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn stop_returns_not_running_when_not_running() {
        let executor = Arc::new(JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init()));
        assert!(executor.stop(true).await.is_err());
    }

    #[tokio::test]
    async fn executor_can_be_restarted_after_stop() {
        let executor = Arc::new(JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init()));
        let handle = executor.run().unwrap();
        executor.stop(true).await.unwrap();
        let _ = handle.await;

        let handle = executor.run().unwrap();
        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn should_add_with_explicit_scheduler() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job_with_scheduler(
                Scheduler::Never,
                Job::new(G, "n", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_register_a_schedule_by_vec() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &vec!["0 1 * * * * *"],
                Job::new(G, "n1", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
            )
            .await
            .unwrap();
        executor
            .add_job(
                &vec!["0 1 * * * * *".to_owned(), "0 1 * * * * *".to_owned()],
                Job::new(G, "n2", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn failing_job_keeps_schedule_due() {
        struct FailingTask;
        impl ScheduledTask<MemoryScheduleRepository> for FailingTask {
            type Error = std::io::Error;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                Err(std::io::Error::other("boom"))
            }
        }

        let repo = MemoryScheduleRepository::init();
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "fail", None, FailingTask),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 0);

        let mut tx = repo.begin().await.unwrap();
        let row = repo.try_claim_due(&mut tx, G, "fail").await.unwrap();
        assert!(row.is_some(), "schedule must remain due after task failure");
        repo.commit(tx).await.unwrap();
    }

    #[tokio::test]
    async fn closure_job_via_from_fn_fires() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::from_fn(G, "via-fn", None, move |_tx| {
                    let count = Arc::clone(&count_clone);
                    Box::pin(async move {
                        count.fetch_add(1, Ordering::SeqCst);
                        Ok::<(), std::io::Error>(())
                    })
                }),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn job_retries_after_failure() {
        struct FlakyTask {
            count: Arc<AtomicUsize>,
            succeed_at: usize,
        }
        impl ScheduledTask<MemoryScheduleRepository> for FlakyTask {
            type Error = std::io::Error;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                let prev = self.count.fetch_add(1, Ordering::SeqCst);
                if prev + 1 >= self.succeed_at {
                    Ok(())
                } else {
                    Err(std::io::Error::other("transient"))
                }
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(
                    G,
                    "flaky",
                    Some(5),
                    FlakyTask { count: Arc::clone(&count), succeed_at: 3 },
                ),
            )
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 1);
        // Failed twice (counts 1 and 2), succeeded on attempt 3.
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    /// Ported from the old `lightspeed_scheduler` test
    /// `should_not_run_an_already_running_job`: while one tick is in flight
    /// holding the row lock, concurrent ticks on the same row must skip.
    #[tokio::test]
    async fn single_executor_does_not_double_fire_a_running_job() {
        struct SlowTask {
            count: Arc<AtomicUsize>,
        }
        impl ScheduledTask<MemoryScheduleRepository> for SlowTask {
            type Error = Infallible;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                tokio::time::sleep(Duration::from_millis(100)).await;
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let executor =
            Arc::new(JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init()));
        // 1h interval + execute_at_startup ⇒ row is due once at registration,
        // then 1h in the future after the first fire commits.
        executor
            .add_job_with_scheduler(
                Scheduler::Interval {
                    interval_duration: Duration::from_secs(3600),
                    execute_at_startup: true,
                },
                Job::new(G, "slow", None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        // Fire 50 concurrent ticks. The row lock (memory mutex / FOR UPDATE
        // SKIP LOCKED on postgres) must let only one tick_one claim the row.
        let mut handles = Vec::with_capacity(50);
        for _ in 0..50 {
            let e = Arc::clone(&executor);
            handles.push(tokio::spawn(async move { e.tick().await.unwrap() }));
        }
        let mut total = 0;
        for h in handles {
            total += h.await.unwrap();
        }

        assert_eq!(total, 1, "exactly one tick should have fired");
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    /// Ported from the old `lightspeed_scheduler` test
    /// `a_running_job_should_not_block_the_executor`: due jobs fire in
    /// parallel inside `tick`, so total wall time tracks the slowest job
    /// rather than the sum.
    #[tokio::test]
    async fn tick_runs_due_jobs_in_parallel() {
        struct SlowTask {
            count: Arc<AtomicUsize>,
        }
        impl ScheduledTask<MemoryScheduleRepository> for SlowTask {
            type Error = Infallible;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                tokio::time::sleep(Duration::from_millis(300)).await;
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let n: usize = 5;
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        for i in 0..n {
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval {
                        interval_duration: Duration::from_secs(3600),
                        execute_at_startup: true,
                    },
                    Job::new(
                        G,
                        format!("slow-{i}"),
                        None,
                        SlowTask { count: Arc::clone(&count) },
                    ),
                )
                .await
                .unwrap();
        }

        let start = std::time::Instant::now();
        let fired = executor.tick().await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(fired, n);
        assert_eq!(count.load(Ordering::SeqCst), n);
        // Parallel: total wall time ~one job. Serial would be ~n*300ms = 1500ms;
        // 900ms catches the regression while tolerating CI jitter.
        assert!(
            elapsed < Duration::from_millis(900),
            "expected parallel execution (~300ms), took {elapsed:?}",
        );
    }

    /// Ported from the old `lightspeed_scheduler` test
    /// `should_gracefully_shutdown_the_job_executor`: `stop(true)` must wait
    /// for every in-flight job to complete, not just the run loop.
    #[tokio::test]
    async fn graceful_stop_waits_for_all_in_flight_jobs() {
        struct SlowTask {
            count: Arc<AtomicUsize>,
        }
        impl ScheduledTask<MemoryScheduleRepository> for SlowTask {
            type Error = Infallible;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                tokio::time::sleep(Duration::from_millis(150)).await;
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let n: usize = 20;
        let count = Arc::new(AtomicUsize::new(0));
        let executor =
            Arc::new(JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init()));
        for i in 0..n {
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval {
                        interval_duration: Duration::from_secs(3600),
                        execute_at_startup: true,
                    },
                    Job::new(
                        G,
                        format!("slow-{i}"),
                        None,
                        SlowTask { count: Arc::clone(&count) },
                    ),
                )
                .await
                .unwrap();
        }

        let handle = executor.run().unwrap();
        // Let the run loop enter `tick` and spawn the per-job tasks before
        // we ask it to stop.
        tokio::time::sleep(Duration::from_millis(20)).await;
        executor.stop(true).await.unwrap();
        let _ = handle.await;

        assert_eq!(count.load(Ordering::SeqCst), n);
    }

    /// Re-registering the same `(group, name)` with a different schedule
    /// definition (different fingerprint) re-anchors the persisted
    /// `next_run_at` instead of leaving the old schedule's anchor in place.
    /// Re-registering with the **same** definition is idempotent.
    #[tokio::test]
    async fn changing_schedule_definition_re_anchors_persisted_next_run_at() {
        let repo = MemoryScheduleRepository::init();

        // First deploy: 1h interval, no startup fire. Row gets a future
        // next_run_at and is not due.
        {
            let executor = JobExecutor::new_with_utc_tz(repo.clone());
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval {
                        interval_duration: Duration::from_secs(3600),
                        execute_at_startup: false,
                    },
                    Job::new(
                        G,
                        "schedule-swap",
                        None,
                        CountingTask { count: Arc::new(AtomicUsize::new(0)) },
                    ),
                )
                .await
                .unwrap();
        }

        // Sanity: row is registered but not due (next_run_at is ~1h out).
        {
            let mut tx = repo.begin().await.unwrap();
            assert!(
                repo.try_claim_due(&mut tx, G, "schedule-swap").await.unwrap().is_none(),
                "1h interval should not be due immediately after register",
            );
            repo.commit(tx).await.unwrap();
        }

        // Second deploy: same name, different schedule definition (now with
        // execute_at_startup = true). The fingerprint differs, so register
        // must re-anchor next_run_at — the new schedule says "due now".
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(
                Scheduler::Interval {
                    interval_duration: Duration::from_secs(3600),
                    execute_at_startup: true,
                },
                Job::new(
                    G,
                    "schedule-swap",
                    None,
                    CountingTask { count: Arc::clone(&count) },
                ),
            )
            .await
            .unwrap();

        // The re-anchored row should fire on the next tick.
        assert_eq!(executor.tick().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    /// Re-registering with the **same** schedule definition (same
    /// fingerprint) must leave the persisted `next_run_at` alone — that's
    /// the idempotency contract.
    #[tokio::test]
    async fn re_registering_same_schedule_is_idempotent() {
        let repo = MemoryScheduleRepository::init();
        // First deploy: execute_at_startup = true ⇒ row is due immediately.
        {
            let executor = JobExecutor::new_with_utc_tz(repo.clone());
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval {
                        interval_duration: Duration::from_secs(3600),
                        execute_at_startup: true,
                    },
                    Job::new(
                        G,
                        "idempotent",
                        None,
                        CountingTask { count: Arc::new(AtomicUsize::new(0)) },
                    ),
                )
                .await
                .unwrap();
            // Fire once, advancing next_run_at to ~1h from now.
            assert_eq!(executor.tick().await.unwrap(), 1);
        }

        // Second "deploy" with the same definition: a freshly-constructed
        // `Scheduler::Interval { 3600s, execute_at_startup: true }` has the
        // same fingerprint as the original, so `register` must NOT re-anchor
        // back to "due now" — otherwise the task would re-fire on every
        // restart.
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(
                Scheduler::Interval {
                    interval_duration: Duration::from_secs(3600),
                    execute_at_startup: true,
                },
                Job::new(
                    G,
                    "idempotent",
                    None,
                    CountingTask { count: Arc::clone(&count) },
                ),
            )
            .await
            .unwrap();

        // Row's persisted next_run_at is ~1h in the future → no fire.
        assert_eq!(executor.tick().await.unwrap(), 0);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    /// Ported from the old `lightspeed_scheduler` test
    /// `job_should_retry_run_if_error`: a task that always errors is
    /// attempted exactly `retries_after_failure + 1` times before the
    /// schedule is rolled back.
    #[tokio::test]
    async fn task_that_always_fails_attempts_retries_plus_one_times() {
        struct AlwaysFailTask {
            count: Arc<AtomicUsize>,
        }
        impl ScheduledTask<MemoryScheduleRepository> for AlwaysFailTask {
            type Error = std::io::Error;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Err(std::io::Error::other("always fails"))
            }
        }

        let retries: u64 = 12;
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(
                    G,
                    "always-fails",
                    Some(retries),
                    AlwaysFailTask { count: Arc::clone(&count) },
                ),
            )
            .await
            .unwrap();

        // All retries exhausted ⇒ no successful firings.
        assert_eq!(executor.tick().await.unwrap(), 0);
        // Initial attempt + `retries` retries.
        assert_eq!(count.load(Ordering::SeqCst), (retries + 1) as usize);
    }
}
