#![doc = include_str!("../README.md")]
// No `unsafe` in this crate.
#![forbid(unsafe_code)]
// `.unwrap()` and `.expect()` are banned in production code.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

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
use std::sync::{Arc, Weak};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Utc;
use chrono_tz::{Tz, UTC};
use log::{debug, info, warn};
use parking_lot::Mutex;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::task::{AbortHandle, JoinHandle, JoinSet};

use crate::scheduler::utc_to_system_time;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type-erased view of a [`ScheduledTask`].
trait ErasedTask<R: ScheduleRepository>: Send + Sync + 'static {
    fn run<'a>(&'a self, tx: &'a mut R::Tx) -> BoxedFuture<'a, Result<(), BoxedError>>;
}

/// Adapter to convert a [`ScheduledTask`] into an [`ErasedTask`].
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

/// Internal representation of a scheduled task.
///
/// `group` / `name` are `Arc<str>` so cloning an `Entry` (which happens on
/// every poll for every due job, via [`JobExecutorInner::tick`]) is just
/// refcount bumps — no `String` allocations on the hot path.
#[derive(Clone)]
struct Entry<R: ScheduleRepository> {
    group: Arc<str>,
    name: Arc<str>,
    schedule: Arc<Mutex<Box<dyn Schedule>>>,
    task: Arc<dyn ErasedTask<R>>,
    retries_after_failure: Option<u64>,
}

/// Internal state for [`JobExecutor::run`].
struct RunningState {
    stop_request: Arc<Notify>,
    finished_rx: oneshot::Receiver<()>,
    abort_handle: AbortHandle,
}

/// Distributed executor for named jobs.
///
/// Register jobs with [`add_job`](Self::add_job) (or
/// [`add_job_with_scheduler`](Self::add_job_with_scheduler) /
/// [`add_job_with_multi_schedule`](Self::add_job_with_multi_schedule)), then
/// call [`run`](Self::run) to spawn the polling loop as a tokio task. The loop
/// stops with [`stop`](Self::stop), or when the last `JobExecutor` handle is
/// dropped: it doesn't count as a handle itself, so it never keeps the
/// executor, or its repository, alive on its own.
#[derive(Clone)]
pub struct JobExecutor<R: ScheduleRepository> {
    inner: Arc<JobExecutorInner<R>>,
}

/// Internal state for [`JobExecutor`].
/// This is shared across all `JobExecutor` cloned instances.
struct JobExecutorInner<R: ScheduleRepository> {
    repo: R,
    timezone: Option<Tz>,
    jobs: Mutex<Vec<Entry<R>>>,
    /// `(group, name)` view of `jobs`, kept in sync by `add_job_with_scheduler`.
    /// `next_sleep_duration` borrows from this directly so it doesn't need to
    /// re-walk `jobs` (and re-allocate two `Vec`s of strings) on every poll
    /// cycle. The `Arc` lets readers snapshot lock-free.
    cached_keys: KeysCell,
    running: AtomicBool,
    state: Mutex<Option<RunningState>>,
    /// Signalled by `add_job` so a sleeping run loop wakes up promptly when a
    /// new job might be due sooner than its current sleep target. Shared with
    /// the loop, which waits on it without holding the executor.
    wake: Arc<Notify>,
}

impl<R: ScheduleRepository> Drop for JobExecutorInner<R> {
    fn drop(&mut self) {
        // The last handle is gone: wake a running loop so it notices and exits
        // instead of sleeping until its next firing.
        if let Some(state) = self.state.get_mut().take() {
            state.stop_request.notify_one();
        }
    }
}

/// Lock-light snapshot of the `(group, name)` pairs of every registered
/// job. Wraps the `Arc` so reading from the run loop is one mutex lock + one
/// `Arc::clone` (refcount bump) — no allocations.
struct KeysCell {
    inner: Mutex<Arc<KeysView>>,
}

type KeysView = Vec<(Arc<str>, Arc<str>)>;

impl KeysCell {
    fn new() -> Self {
        Self { inner: Mutex::new(Arc::new(Vec::new())) }
    }

    fn load(&self) -> Arc<KeysView> {
        Arc::clone(&self.inner.lock())
    }

    fn store(&self, view: Arc<KeysView>) {
        *self.inner.lock() = view;
    }
}

/// Rebuilds the cached `(group, name)` view from the current `jobs` vec.
/// Called from `add_job` (cold path); the run loop never calls this.
fn build_keys_view<R: ScheduleRepository>(jobs: &[Entry<R>]) -> KeysView {
    jobs.iter().map(|e| (Arc::clone(&e.group), Arc::clone(&e.name))).collect()
}

/// Upper bound on a single sleep interval.
const MAX_SLEEP: Duration = Duration::from_secs(3600);

/// Pause before polling again when a job is due but the tick fired nothing:
/// another process holds its row, or its task failed. Without it the loop
/// would retry right away, over and over. Other jobs coming due meanwhile
/// fire when the pause ends. It is also the wait when the next due time
/// can't be read.
const RETRY_DELAY: Duration = Duration::from_secs(1);

/// Far-future sentinel (year 9999) used for [`Scheduler::Never`] schedules
/// whose `next()` returns `None` — the row is registered but will never
/// become due.
#[inline]
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
            inner: Arc::new(JobExecutorInner {
                repo,
                timezone,
                jobs: Mutex::new(Vec::new()),
                cached_keys: KeysCell::new(),
                running: AtomicBool::new(false),
                state: Mutex::new(None),
                wake: Arc::new(Notify::new()),
            }),
        }
    }

    /// Spawns a tokio task that drives the executor and returns its
    /// `JoinHandle`. Besides [`stop`](Self::stop), the loop exits when the last
    /// `JobExecutor` handle is dropped, after finishing the tick in progress,
    /// if any.
    ///
    /// When a due job doesn't fire, because another process holds it or its
    /// task failed, the loop waits a second before polling again.
    pub fn run(&self) -> Result<JoinHandle<()>, SchedulerError> {
        let mut state_guard = self.inner.state.lock();
        if self.inner.running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(SchedulerError::JobExecutionStateError {
                message: "The JobExecutor is already running.".to_owned(),
            });
        }

        let stop_request = Arc::new(Notify::new());
        let (finished_tx, finished_rx) = oneshot::channel();
        let me_stop = Arc::clone(&stop_request);

        let cleanup = RunCleanup { me: Arc::downgrade(&self.inner), finished_tx: Some(finished_tx) };

        let me = Arc::downgrade(&self.inner);
        let wake = Arc::clone(&self.inner.wake);
        let job_count = self.inner.jobs.lock().len();
        let handle = tokio::spawn(async move {
            let _cleanup = cleanup;

            info!(
                target: "lightspeed_scheduler",
                "JobExecutor started. jobs: {job_count}",
            );
            // The executor is only held while an iteration runs, so dropping
            // its last handle ends the loop instead of being kept alive by it.
            while let Some(inner) = me.upgrade() {
                let fired = match inner.tick().await {
                    Ok(fired) => fired,
                    Err(e) => {
                        warn!(target: "lightspeed_scheduler", "tick failed. error: {e}");
                        0
                    }
                };

                let mut sleep_for = inner.next_sleep_duration().await;
                if fired == 0 && sleep_for.is_zero() {
                    sleep_for = RETRY_DELAY;
                }
                drop(inner);
                tokio::select! {
                    _ = me_stop.notified() => break,
                    // `add_job` rang the bell — re-tick and recompute the sleep.
                    _ = wake.notified() => {}
                    _ = tokio::time::sleep(sleep_for) => {}
                }
            }
            info!(target: "lightspeed_scheduler", "JobExecutor stopped");
        });

        *state_guard = Some(RunningState { stop_request, finished_rx, abort_handle: handle.abort_handle() });
        drop(state_guard);

        Ok(handle)
    }
}

// Thin pass-through delegation to the shared inner state. Keeps
// `JobExecutorInner` private — callers only ever see the `JobExecutor` handle.
impl<R: ScheduleRepository> JobExecutor<R> {
    /// Adds a job using any [`TryToScheduler`]-convertible schedule (cron
    /// string, `Duration`, etc.).
    pub async fn add_job<T>(&self, schedule: &dyn TryToScheduler, job: Job<T>) -> Result<(), SchedulerError>
    where
        T: ScheduledTask<R>,
    {
        self.inner.add_job(schedule, job).await
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
        self.inner.add_job_with_multi_schedule(schedule, job).await
    }

    /// Adds a job with an explicit [`Scheduler`] (or anything `Into<Scheduler>`).
    pub async fn add_job_with_scheduler<S, T>(&self, schedule: S, job: Job<T>) -> Result<(), SchedulerError>
    where
        S: Into<Scheduler>,
        T: ScheduledTask<R>,
    {
        self.inner.add_job_with_scheduler(schedule, job).await
    }

    /// Walks every registered job once, firing those that are due in
    /// parallel.
    /// Returns the number of jobs that fired successfully this cycle.
    ///
    /// Returns once every job it started has finished. A repository error
    /// from any of them is returned after that, the first one if several
    /// failed. Dropping the future aborts the jobs and rolls their
    /// transactions back.
    pub async fn tick(&self) -> Result<usize, SchedulerError> {
        self.inner.tick().await
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
        self.inner.stop(graceful).await
    }
}

impl<R: ScheduleRepository> JobExecutorInner<R> {
    /// Adds a job using any [`TryToScheduler`]-convertible schedule (cron
    /// string, `Duration`, etc.).
    async fn add_job<T>(&self, schedule: &dyn TryToScheduler, job: Job<T>) -> Result<(), SchedulerError>
    where
        T: ScheduledTask<R>,
    {
        let scheduler = schedule.to_scheduler()?;
        self.add_job_with_scheduler(scheduler, job).await
    }

    /// Adds a job whose schedule fires whenever any of the given expressions
    /// fires.
    async fn add_job_with_multi_schedule<T>(
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
    async fn add_job_with_scheduler<S, T>(&self, schedule: S, job: Job<T>) -> Result<(), SchedulerError>
    where
        S: Into<Scheduler>,
        T: ScheduledTask<R>,
    {
        let mut scheduler: Scheduler = schedule.into();
        let Job { group, name, retries_after_failure, task } = job;
        let group: Arc<str> = Arc::from(group);
        let name: Arc<str> = Arc::from(name);

        let fingerprint = scheduler.fingerprint();

        let now_utc = Utc::now();
        let next_system = match scheduler.next(&now_utc, self.timezone) {
            Some(next_utc) => utc_to_system_time(next_utc),
            None => never_sentinel(),
        };
        self.repo.register(&group, &name, next_system, &fingerprint).await?;

        info!(target: "lightspeed_scheduler", "add job to executor. group: {group}, name: {name}");
        let boxed: Box<dyn Schedule> = Box::new(scheduler);
        let entry = Entry {
            group,
            name,
            schedule: Arc::new(Mutex::new(boxed)),
            task: Arc::new(TaskAdapter(task)),
            retries_after_failure,
        };
        {
            let mut jobs = self.jobs.lock();
            jobs.push(entry);
            self.cached_keys.store(Arc::new(build_keys_view(&jobs)));
        }
        self.wake.notify_one();
        Ok(())
    }

    /// Time to sleep before the next iteration, derived from the persisted
    /// `next_run_at` of every registered job. Capped at [`MAX_SLEEP`].
    /// [`RETRY_DELAY`] when the repository can't be read, so that a database
    /// outage doesn't stall the loop for an hour.
    async fn next_sleep_duration(&self) -> Duration {
        let view = self.cached_keys.load();
        let refs: Vec<(&str, &str)> = view.iter().map(|(g, n)| (g.as_ref(), n.as_ref())).collect();
        match self.repo.time_until_next_due(&refs).await {
            Ok(until) => until.unwrap_or(MAX_SLEEP).min(MAX_SLEEP),
            Err(e) => {
                warn!(
                    target: "lightspeed_scheduler",
                    "failed to read next due time; retrying in {RETRY_DELAY:?}. error: {e}",
                );
                RETRY_DELAY
            }
        }
    }

    /// Walks every registered job once, firing in parallel those the
    /// repository reports as due.
    async fn tick(&self) -> Result<usize, SchedulerError> {
        let view = self.cached_keys.load();
        let refs: Vec<(&str, &str)> = view.iter().map(|(g, n)| (g.as_ref(), n.as_ref())).collect();
        let due = self.repo.due_keys(&refs).await?;
        if due.is_empty() {
            return Ok(0);
        }
        let snapshot: Vec<Entry<R>> = {
            let jobs = self.jobs.lock();
            jobs.iter()
                .filter(|e| due.iter().any(|(g, n)| g.as_str() == &*e.group && n.as_str() == &*e.name))
                .cloned()
                .collect()
        };
        // Dropping a JoinSet aborts the jobs it owns, so cancelling the tick
        // (e.g. `stop(false)`) rolls their transactions back.
        let mut set = JoinSet::new();
        for entry in snapshot {
            set.spawn(Self::tick_one(self.repo.clone(), self.timezone, entry));
        }
        let mut fired = 0;
        let mut first_err = None;
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(Ok(true)) => fired += 1,
                Ok(Ok(false)) => {}
                Ok(Err(e)) => {
                    first_err.get_or_insert(e);
                }
                Err(join_err) => {
                    warn!(
                        target: "lightspeed_scheduler",
                        "spawned tick task failed: {join_err}",
                    );
                }
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(fired),
        }
    }

    async fn tick_one(repo: R, tz: Option<Tz>, entry: Entry<R>) -> Result<bool, SchedulerError> {
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
            let mut guard = entry.schedule.lock();
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

    /// Stops the running loop and waits for it to exit. Returns
    /// [`SchedulerError::JobExecutionStateError`] when no `run` is active.
    ///
    /// - `graceful = true`: signals the loop to break at its next select!
    ///   boundary, so the in-flight tick (if any) runs to completion.
    /// - `graceful = false`: aborts the spawned task immediately. The
    ///   in-flight tick is cancelled at its next await point; any open
    ///   transaction is rolled back when the connection is returned to the
    ///   pool.
    async fn stop(&self, graceful: bool) -> Result<(), SchedulerError> {
        let state = self.state.lock().take().ok_or_else(|| SchedulerError::JobExecutionStateError {
            message: "The JobExecutor is not running.".to_owned(),
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
    me: Weak<JobExecutorInner<R>>,
    finished_tx: Option<oneshot::Sender<()>>,
}

impl<R: ScheduleRepository> Drop for RunCleanup<R> {
    fn drop(&mut self) {
        // There is nothing to clear if the loop exited because the executor is gone.
        if let Some(inner) = self.me.upgrade() {
            *inner.state.lock() = None;
            inner.running.store(false, Ordering::SeqCst);
        }
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
        async fn run(&self, _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx) -> Result<(), Self::Error> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct SlowTask {
        count: Arc<AtomicUsize>,
    }

    impl<R: ScheduleRepository> ScheduledTask<R> for SlowTask {
        type Error = Infallible;
        async fn run(&self, _tx: &mut R::Tx) -> Result<(), Self::Error> {
            tokio::time::sleep(Duration::from_millis(200)).await;
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
            .add_job(&Duration::from_secs(3600), Job::new(G, "c", None, CountingTask { count: Arc::clone(&count_c) }))
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 2);
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
        assert_eq!(count_c.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn run_returns_already_running_when_called_twice() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        let handle = executor.run().unwrap();
        assert!(executor.inner.running.load(Ordering::SeqCst));

        assert!(executor.run().is_err());

        executor.stop(false).await.unwrap();
        let _ = handle.await;
        assert!(!executor.inner.running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn stop_returns_not_running_when_not_running() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        assert!(executor.stop(true).await.is_err());
    }

    #[tokio::test]
    async fn executor_can_be_restarted_after_stop() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        let handle = executor.run().unwrap();
        executor.stop(true).await.unwrap();
        let _ = handle.await;

        let handle = executor.run().unwrap();
        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn dropping_the_executor_stops_the_loop() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        let handle = executor.run().unwrap();

        drop(executor);

        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("the loop must stop once the executor is dropped")
            .unwrap();
    }

    #[tokio::test]
    async fn dropping_a_clone_keeps_the_loop_running() {
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        let handle = executor.run().unwrap();

        drop(executor.clone());
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(executor.inner.running.load(Ordering::SeqCst));
        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn dropping_the_executor_lets_the_in_flight_tick_finish() {
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            // Always due: if the loop kept running it would fire again.
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "slow", None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        let handle = executor.run().unwrap();
        // Wait long enough for the loop to enter the slow job.
        tokio::time::sleep(Duration::from_millis(50)).await;

        drop(executor);

        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("the loop must stop once the executor is dropped")
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
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
        executor.add_job(&(Duration::from_millis(0), true), Job::new(G, "fail", None, FailingTask)).await.unwrap();

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
                if prev + 1 >= self.succeed_at { Ok(()) } else { Err(std::io::Error::other("transient")) }
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "flaky", Some(5), FlakyTask { count: Arc::clone(&count), succeed_at: 3 }),
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
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        // 1h interval + execute_at_startup ⇒ row is due once at registration,
        // then 1h in the future after the first fire commits.
        executor
            .add_job_with_scheduler(
                Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                Job::new(G, "slow", None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        // Fire 50 concurrent ticks. The row lock (memory mutex / FOR UPDATE
        // SKIP LOCKED on postgres) must let only one tick_one claim the row.
        let mut handles = Vec::with_capacity(50);
        for _ in 0..50 {
            let e = executor.clone();
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
                    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                    Job::new(G, format!("slow-{i}"), None, SlowTask { count: Arc::clone(&count) }),
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
        assert!(elapsed < Duration::from_millis(900), "expected parallel execution (~300ms), took {elapsed:?}",);
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
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        for i in 0..n {
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                    Job::new(G, format!("slow-{i}"), None, SlowTask { count: Arc::clone(&count) }),
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
                    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: false },
                    Job::new(G, "schedule-swap", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
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
                Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                Job::new(G, "schedule-swap", None, CountingTask { count: Arc::clone(&count) }),
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
                    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                    Job::new(G, "idempotent", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
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
                Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                Job::new(G, "idempotent", None, CountingTask { count: Arc::clone(&count) }),
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
                Job::new(G, "always-fails", Some(retries), AlwaysFailTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        // All retries exhausted ⇒ no successful firings.
        assert_eq!(executor.tick().await.unwrap(), 0);
        // Initial attempt + `retries` retries.
        assert_eq!(count.load(Ordering::SeqCst), (retries + 1) as usize);
    }

    #[tokio::test]
    async fn run_loop_pauses_while_another_process_holds_a_due_job() {
        let repo = MemoryScheduleRepository::init();
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(
                Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                Job::new(G, "held", None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        // Another process claims the due job...
        let mut held = repo.begin().await.unwrap();
        assert!(repo.try_claim_due(&mut held, G, "held").await.unwrap().is_some());

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // ...and gives up without advancing it. A loop retrying right away
        // would take over at once.
        repo.rollback(held).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(count.load(Ordering::SeqCst), 0);

        tokio::time::sleep(RETRY_DELAY).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);

        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn run_loop_pauses_before_retrying_a_failed_job() {
        struct AlwaysFailTask {
            attempts: Arc<AtomicUsize>,
        }
        impl ScheduledTask<MemoryScheduleRepository> for AlwaysFailTask {
            type Error = std::io::Error;
            async fn run(
                &self,
                _tx: &mut <MemoryScheduleRepository as ScheduleRepository>::Tx,
            ) -> Result<(), Self::Error> {
                self.attempts.fetch_add(1, Ordering::SeqCst);
                Err(std::io::Error::other("always fails"))
            }
        }

        let attempts = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "fail", None, AlwaysFailTask { attempts: Arc::clone(&attempts) }),
            )
            .await
            .unwrap();

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let attempted = attempts.load(Ordering::SeqCst);

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(attempts.load(Ordering::SeqCst), attempted, "the failed job must not be retried right away");

        tokio::time::sleep(RETRY_DELAY).await;
        assert!(attempts.load(Ordering::SeqCst) > attempted, "the failed job must be retried after the pause");

        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn restarted_executor_fires_a_missed_firing() {
        let repo = MemoryScheduleRepository::init();
        let schedule =
            || Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: false };

        // A previous deploy registered the job, and its firing time passed
        // while no process was running.
        JobExecutor::new_with_utc_tz(repo.clone())
            .add_job_with_scheduler(
                schedule(),
                Job::new(G, "missed", None, CountingTask { count: Arc::new(AtomicUsize::new(0)) }),
            )
            .await
            .unwrap();
        let now = SystemTime::now();
        let mut tx = repo.begin().await.unwrap();
        repo.advance(&mut tx, G, "missed", now - Duration::from_secs(1), now - Duration::from_secs(3600))
            .await
            .unwrap();
        repo.commit(tx).await.unwrap();

        // The new deploy keeps the persisted firing, so it is due right away.
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(schedule(), Job::new(G, "missed", None, CountingTask { count: Arc::clone(&count) }))
            .await
            .unwrap();

        assert_eq!(executor.tick().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn run_loop_catches_up_without_pausing() {
        let count = Arc::new(AtomicUsize::new(0));
        let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());
        // Still due after each firing, like a job that is behind.
        executor
            .add_job(
                &(Duration::from_millis(0), true),
                Job::new(G, "behind", None, CountingTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        executor.stop(true).await.unwrap();
        let _ = handle.await;

        let fired = count.load(Ordering::SeqCst);
        assert!(fired > 10, "expected back-to-back firings, got {fired}");
    }

    /// Memory repository that fails where a test asks it to, as when the
    /// database is down.
    #[derive(Clone, Default)]
    struct FaultyRepo {
        inner: MemoryScheduleRepository,
        /// Fail every read of the next due time.
        fail_reads: bool,
        /// Reads of the next due time: the run loop makes one per iteration.
        reads: Arc<AtomicUsize>,
        /// Fail every claim of the job with this name.
        fail_claims_of: Option<&'static str>,
    }

    impl ScheduleRepository for FaultyRepo {
        type Tx = <MemoryScheduleRepository as ScheduleRepository>::Tx;

        async fn begin(&self) -> Result<Self::Tx, SchedulerError> {
            self.inner.begin().await
        }

        async fn commit(&self, tx: Self::Tx) -> Result<(), SchedulerError> {
            self.inner.commit(tx).await
        }

        async fn rollback(&self, tx: Self::Tx) -> Result<(), SchedulerError> {
            self.inner.rollback(tx).await
        }

        async fn register(
            &self,
            group: &str,
            name: &str,
            next_run_at: SystemTime,
            schedule_fingerprint: &str,
        ) -> Result<(), SchedulerError> {
            self.inner.register(group, name, next_run_at, schedule_fingerprint).await
        }

        async fn try_claim_due(
            &self,
            tx: &mut Self::Tx,
            group: &str,
            name: &str,
        ) -> Result<Option<ScheduleRow>, SchedulerError> {
            if self.fail_claims_of == Some(name) {
                return Err(SchedulerError::JobExecutionStateError { message: "database unavailable".to_owned() });
            }
            self.inner.try_claim_due(tx, group, name).await
        }

        async fn due_keys(&self, keys: &[(&str, &str)]) -> Result<Vec<(String, String)>, SchedulerError> {
            self.inner.due_keys(keys).await
        }

        async fn advance(
            &self,
            tx: &mut Self::Tx,
            group: &str,
            name: &str,
            next_run_at: SystemTime,
            last_run_at: SystemTime,
        ) -> Result<(), SchedulerError> {
            self.inner.advance(tx, group, name, next_run_at, last_run_at).await
        }

        async fn time_until_next_due(&self, keys: &[(&str, &str)]) -> Result<Option<Duration>, SchedulerError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.fail_reads {
                return Err(SchedulerError::JobExecutionStateError { message: "database unavailable".to_owned() });
            }
            self.inner.time_until_next_due(keys).await
        }
    }

    #[tokio::test]
    async fn run_loop_retries_soon_when_the_next_due_time_cannot_be_read() {
        let repo = FaultyRepo { fail_reads: true, ..Default::default() };
        let executor = JobExecutor::new_with_utc_tz(repo.clone());

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let read = repo.reads.load(Ordering::SeqCst);

        tokio::time::sleep(RETRY_DELAY + Duration::from_millis(200)).await;
        assert!(repo.reads.load(Ordering::SeqCst) > read, "the loop must read the next due time again after the pause");

        executor.stop(true).await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn graceful_stop_waits_for_in_flight_jobs_after_a_repository_error() {
        let count = Arc::new(AtomicUsize::new(0));
        let executor =
            JobExecutor::new_with_utc_tz(FaultyRepo { fail_claims_of: Some("broken"), ..Default::default() });
        // Claiming `broken` fails at once, while `slow` is still running.
        for (name, count) in [("broken", Arc::new(AtomicUsize::new(0))), ("slow", Arc::clone(&count))] {
            executor
                .add_job_with_scheduler(
                    Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                    Job::new(G, name, None, SlowTask { count }),
                )
                .await
                .unwrap();
        }

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        executor.stop(true).await.unwrap();
        let _ = handle.await;

        assert_eq!(count.load(Ordering::SeqCst), 1, "stop(true) must wait for the job still running");
    }

    #[tokio::test]
    async fn non_graceful_stop_rolls_back_in_flight_jobs() {
        let count = Arc::new(AtomicUsize::new(0));
        let repo = MemoryScheduleRepository::init();
        let executor = JobExecutor::new_with_utc_tz(repo.clone());
        executor
            .add_job_with_scheduler(
                Scheduler::Interval { interval_duration: Duration::from_secs(3600), execute_at_startup: true },
                Job::new(G, "slow", None, SlowTask { count: Arc::clone(&count) }),
            )
            .await
            .unwrap();

        let handle = executor.run().unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        executor.stop(false).await.unwrap();
        let _ = handle.await;
        // Past the point where the job would have finished.
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert_eq!(count.load(Ordering::SeqCst), 0, "the job must be aborted");
        let mut tx = repo.begin().await.unwrap();
        assert!(
            repo.try_claim_due(&mut tx, G, "slow").await.unwrap().is_some(),
            "the aborted job must be rolled back, staying due and claimable",
        );
        repo.commit(tx).await.unwrap();
    }
}
