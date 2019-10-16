use chrono::prelude::Local;
use uuid::Uuid;
use rand::Rng;
use rand::distributions::Alphanumeric;

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

pub fn random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect::<String>()
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_return_a_random_string() {
        assert_eq!(10, random_string(10).len());
        assert_eq!(0, random_string(0).len());
    }
}