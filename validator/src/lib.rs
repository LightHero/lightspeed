#![doc = include_str!("../README.md")]

mod error;
mod validation;

pub use error::*;
pub use lightspeed_validator_derive::Validable;
pub use validation::*;
