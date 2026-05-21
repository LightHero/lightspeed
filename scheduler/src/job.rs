use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::repository::ScheduleRepository;

/// User-supplied work invoked when a [`Job`] fires.
///
/// The handler runs inside the same transaction that holds the lock on the
/// schedule row. Returning `Ok(())` commits the advance; returning an error
/// rolls the transaction back so the schedule remains due for the next poll.
pub trait ScheduledTask<R: ScheduleRepository>: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    fn run(&self, tx: &mut R::Tx) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Boxed future a closure handed to [`Job::from_fn`] must return. The
/// lifetime `'a` ties the future to the borrow of the transaction so the
/// closure body can `.await` on operations that hold `tx`.
pub type TxBoxFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;

/// Adapts a closure to the [`ScheduledTask`] trait. Built via
/// [`Job::from_fn`]; you don't normally reference this type by name.
pub struct FnTask<F, R, E> {
    f: F,
    _marker: PhantomData<fn() -> (R, E)>,
}

impl<F, R, E> ScheduledTask<R> for FnTask<F, R, E>
where
    R: ScheduleRepository,
    F: for<'a> Fn(&'a mut R::Tx) -> TxBoxFuture<'a, E> + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = E;
    fn run(&self, tx: &mut R::Tx) -> impl Future<Output = Result<(), Self::Error>> + Send {
        (self.f)(tx)
    }
}

/// A scheduled unit of work paired with its identifying `(group, name)` and
/// optional retry policy. Mirrors the `Job` type from the old
/// `lightspeed_scheduler` crate so existing call sites stay close.
///
/// Construct with [`Job::new`] (for any `T: ScheduledTask<R>`) or
/// [`Job::from_fn`] (for a closure). Hand the result to
/// [`JobExecutor::add_job`](crate::JobExecutor::add_job) or one of its
/// variants.
pub struct Job<T> {
    pub group: String,
    pub name: String,
    /// Number of additional attempts after an initial failure. `None` means
    /// no retries — the task runs once.
    pub retries_after_failure: Option<u64>,
    pub task: T,
}

impl<T> Job<T> {
    pub fn new(
        group: impl Into<String>,
        name: impl Into<String>,
        retries_after_failure: Option<u64>,
        task: T,
    ) -> Self {
        Self { group: group.into(), name: name.into(), retries_after_failure, task }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group(&self) -> &str {
        &self.group
    }
}

impl<F, R, E> Job<FnTask<F, R, E>>
where
    R: ScheduleRepository,
    F: for<'a> Fn(&'a mut R::Tx) -> TxBoxFuture<'a, E> + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Builds a job from a closure that returns a boxed future. The closure
    /// receives the same `&mut R::Tx` a [`ScheduledTask::run`] impl would,
    /// so user work runs in the transaction that holds the schedule row.
    ///
    /// ```ignore
    /// Job::from_fn("group", "name", None, |_tx| Box::pin(async move {
    ///     // do something
    ///     Ok::<(), std::io::Error>(())
    /// }));
    /// ```
    pub fn from_fn(
        group: impl Into<String>,
        name: impl Into<String>,
        retries_after_failure: Option<u64>,
        f: F,
    ) -> Self {
        Self {
            group: group.into(),
            name: name.into(),
            retries_after_failure,
            task: FnTask { f, _marker: PhantomData },
        }
    }
}
