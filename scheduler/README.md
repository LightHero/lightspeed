# lightspeed-scheduler

An in-process scheduler for periodic jobs. Schedule lets you run Rust functions on a cron-like schedule.


## Usage

```rust
use schedule::Agenda;
use chrono::Utc;

fn main() {
    let mut a = Agenda::new();

    // Run every second
    a.add(|| {
        println!("at second     :: {}", Utc::now());
        Ok(())
    }).schedule("* * * * * *").unwrap();

    // Run every minute
    a.add(|| {
        println!("at minute     :: {}", Utc::now());
        Ok(())
    }).schedule("0 * * * * *").unwrap();

    // Run every hour
    a.add(|| {
        println!("at hour       :: {}", Utc::now());
        Ok(())
    }).schedule("0 0 * * * *").unwrap();

    // Check and run pending jobs in agenda every 500 milliseconds
    loop {
        a.run_pending();

        std::thread::sleep(std::time::Duration::from_millis(500));
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
