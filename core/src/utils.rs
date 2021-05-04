use chrono::prelude::Local;
use uuid::Uuid;

/// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
/// (aka "UNIX timestamp").
#[inline]
pub fn current_epoch_seconds() -> i64 {
    Local::now().timestamp()
}

#[inline]
pub fn new_hyphenated_uuid() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}
