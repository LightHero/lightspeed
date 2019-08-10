# lightspeed-scheduler

An in-process scheduler for periodic jobs. Schedule lets you run Rust functions on a cron-like schedule.


## Usage

```rust
extern crate schedule;
extern crate chrono;

use schedule::Agenda;
use chrono::UTC;

fn main() {
    let mut a = Agenda::new();

    // Run every second
    a.add(|| {
        println!("at second     :: {}", UTC::now());
    }).schedule("* * * * * *").unwrap();

    // Run every minute
    a.add(|| {
        println!("at minute     :: {}", UTC::now());
    }).schedule("0 * * * * *").unwrap();

    // Run every hour
    a.add(|| {
        println!("at hour       :: {}", UTC::now());
    }).schedule("0 0 * * * *").unwrap();

    // Check and run pending jobs in agenda every 500 milliseconds
    loop {
        a.run_pending();

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
```

## License

Schedule is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.


## Credits

Originally based on https://github.com/mehcode/schedule-rs
