# lightspeed-scheduler

A Rust scheduler for periodic jobs on a cron-like schedule. In-process by
default and **optionally distributed** with a Postgres, MySQL, or SQLite
backend: **exactly one process fires each scheduled tick of a given job**
across all replicas.


## Usage

```rust,no_run
use std::time::Duration;
use lightspeed_scheduler::{
    Job, JobExecutor, MemoryScheduleRepository, Scheduler, TryToScheduler,
};

#[tokio::main]
async fn main() {
    // `MemoryScheduleRepository` is the in-process backend. For a
    // distributed deployment, swap in `PgScheduleRepository::init(c3p0).await?`
    // (or the `MySqlScheduleRepository` / `SqliteScheduleRepository`
    // equivalent) so every process in the cluster shares the same
    // coordination state.
    //
    // `JobExecutor` is a cheap `Arc`-shared handle internally — clone it
    // freely to hand a copy to other tasks; no outer `Arc::new` wrap needed.
    let executor = JobExecutor::new_with_utc_tz(MemoryScheduleRepository::init());

    // Run every 10 seconds with no retries in case of failure
    let retries = None;
    executor
        .add_job_with_scheduler(
            Scheduler::Interval {
                interval_duration: Duration::from_secs(10),
                execute_at_startup: true,
            },
            Job::from_fn("hello_job", "job_1", retries, |_tx| {
                Box::pin(async move {
                    println!("Hello from job. This happens every 10 seconds!");
                    Ok::<(), std::io::Error>(())
                })
            }),
        )
        .await
        .unwrap();

    // Run every day at 2:00 am with three retries in case of failure
    let retries = Some(3);
    executor
        .add_job_with_scheduler(
            "0 0 2 * * *".to_scheduler().unwrap(),
            Job::from_fn("hello_job", "job_2", retries, |_tx| {
                Box::pin(async move {
                    println!("Hello from job. This happens every day at 2 am!");
                    Ok::<(), std::io::Error>(())
                })
            }),
        )
        .await
        .unwrap();

    // Start the job executor
    let _executor_handle = executor.run().expect("The job executor should run!");

    // In a real app, await a shutdown signal here.

    // Stop the job executor
    let stop_gracefully = true;
    executor.stop(stop_gracefully).await.expect("The job executor should stop!");
}
```

The closure receives `&mut R::Tx` so user work runs in the same transaction
that holds the schedule row's lock — work commits atomically with the
schedule advance, and a returned `Err` rolls back so the schedule remains
due for the next poll. If you don't need the transaction, ignore the `_tx`
argument as above.

In case of database repositories, the transaction is the one that holds the
lock on the schedule row.

For more elaborate jobs, implement the [`ScheduledTask`] trait directly and
pass it to [`Job::new`] instead of [`Job::from_fn`].

## Storage backends

The executor takes a `ScheduleRepository`. The choice of backend is what
determines whether the scheduler runs in-process only or coordinates across
multiple processes:

- `MemoryScheduleRepository` — in-process only, non distributed. For tests or a
  single-process deployment.
- `PgScheduleRepository` / `MySqlScheduleRepository` /
  `SqliteScheduleRepository` — SQL-backed. **The Postgres and MySQL
  backends support distributed deployments**: every process in the cluster
  registers the same jobs against the same database; row-level locking
  picks exactly one process to fire each tick. SQLite is single-process by
  design.

Distributed safety on Postgres/MySQL relies on `FOR UPDATE SKIP LOCKED`:
when one process claims a schedule row inside a transaction, concurrent
claimers see the row as locked and skip it. To avoid local time differences, 
the DB time is used to determine the next Task execution time.


```rust,no_run
// Compiled only when the `postgres` feature is enabled; the rest of the
// block is gated out so this doctest compiles regardless.
#[cfg(feature = "postgres")]
mod postgres_example {
    use c3p0::PgC3p0Pool;
    use c3p0::sqlx::PgPool;
    use lightspeed_scheduler::{JobExecutor, PgScheduleRepository};

    pub async fn build() -> Result<(), Box<dyn std::error::Error>> {
        let pool: PgPool = unimplemented!("your application's sqlx pool");
        let c3p0 = PgC3p0Pool::new(pool);
        let repo = PgScheduleRepository::init(c3p0).await?;
        let executor = JobExecutor::new_with_utc_tz(repo);

        // ...Register the jobs with the executor here...

        let _handle = executor.run().expect("The job executor should run!");

        Ok(())
    }
}
```

## Cron schedule format
Creating a schedule for a job is done using the `FromStr` impl for the
`Schedule` type of the [cron](https://github.com/zslayton/cron) library.

The scheduling format is as follows:

```text
sec   min   hour   day of month   month   day of week   year
*     *     *      *              *       *             *
```

Time is specified for `UTC` and not your local timezone. Note that the year may
be omitted.

Comma separated values such as `5,8,10` represent more than one time value. So
for example, a schedule of `0 2,14,26 * * * *` would execute on the 2nd, 14th,
and 26th minute of every hour.

Ranges can be specified with a dash. A schedule of `0 0 * 5-10 * *` would
execute once per hour but only on day 5 through 10 of the month.

Day of the week can be specified as an abbreviation or the full name. A
schedule of `0 0 6 * * Sun,Sat` would execute at 6am on Sunday and Saturday.

## Credits

Originally based on https://github.com/mehcode/schedule-rs
