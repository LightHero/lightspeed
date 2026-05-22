// No `unsafe` in this crate.
#![forbid(unsafe_code)]
// `.unwrap()` and `.expect()` are banned in production code.
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used)
)]

#[cfg(feature = "c3p0")]
pub use c3p0;

#[cfg(feature = "account_management")]
pub use lightspeed_account_management as account_management;

#[cfg(feature = "cache")]
pub use lightspeed_cache as cache;

#[cfg(feature = "core")]
pub use lightspeed_core as core;

#[cfg(feature = "email")]
pub use lightspeed_email as email;

#[cfg(feature = "file_store")]
pub use lightspeed_file_store as file_store;

#[cfg(feature = "hash")]
pub use lightspeed_hash as hash;

#[cfg(feature = "logger")]
pub use lightspeed_logger as logger;

#[cfg(feature = "scheduler")]
pub use lightspeed_scheduler as scheduler;

#[cfg(feature = "validator")]
pub use lightspeed_validator as validator;
