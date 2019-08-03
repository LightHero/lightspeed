use chrono::prelude::Local;

/// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
/// (aka "UNIX timestamp").
#[inline]
pub fn current_epoch_seconds() -> i64 {
    Local::now().timestamp()
}