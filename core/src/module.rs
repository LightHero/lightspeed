use std::future::Future;

use crate::error::LightSpeedError;

pub trait Module {
    fn start(&mut self) -> impl Future<Output = Result<(), LightSpeedError>> + Send;
}
