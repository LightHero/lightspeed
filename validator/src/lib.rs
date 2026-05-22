#![doc = include_str!("../README.md")]
// No `unsafe` in this crate.
#![forbid(unsafe_code)]
// `.unwrap()` and `.expect()` are banned in production code.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod error;
mod validation;

pub use error::*;
pub use lightspeed_validator_derive::Validable;
pub use validation::*;
