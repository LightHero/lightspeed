use std::future::Future;

use crate::error::LsError;

pub trait LsModule {
    fn start(&mut self) -> impl Future<Output = Result<(), LsError>> + Send;
}

