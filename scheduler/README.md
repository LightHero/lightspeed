# lightspeed-scheduler

An in-process scheduler for periodic jobs. Schedule lets you run Rust functions on a cron-like schedule.


## Usage

```rust
    use std::time::Duration;
    use lightspeed_scheduler::{job::Job, scheduler::{Scheduler, TryToScheduler}, JobExecutor};
    
    #[tokio::main]
    async fn main() {
        let executor = JobExecutor::new_with_utc_tz();
    
        // Run every 10 seconds with no retries in case of failure
        let retries = None;
        executor
            .add_job_with_scheduler(
                Scheduler::Interval {
                    interval_duration: Duration::from_secs(10),
                    execute_at_startup: true,
                },
                Job::new("hello_job", "job_1", retries, move || {
                    Box::pin(async move {
                        println!("Hello from job. This happens every 10 seconds!");
                        Ok(())
                    })
                }),
            )
            .await;
    

        // Run every day at 2:00 am with two retries in case of failure
        let retries = Some(2);
        executor
        .add_job_with_scheduler(
            "0 0 2 * * *".to_scheduler().unwrap(),
            Job::new("hello_job", "job_2", retries, move || {
                Box::pin(async move {
                    println!("Hello from job. This happens every day at 2 am!");
                    Ok(())
                })
            }),
        )
        .await;

        // Start the job executor
        let _executor_handle = executor.run().await.expect("The job executor should run!");
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
