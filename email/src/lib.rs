use crate::config::EmailClientConfig;
use crate::service::LsEmailService;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct LsEmailClientModule {
    pub email_config: EmailClientConfig,
    pub email_service: Arc<LsEmailService>,
}

impl LsEmailClientModule {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LsError> {
        println!("Creating LsEmailClientModule");
        info!("Creating LsEmailClientModule");

        let email_service = Arc::new(LsEmailService::new(repository::email::new(email_config.clone())?));

        Ok(LsEmailClientModule { email_config, email_service })
    }
}

impl lightspeed_core::module::LsModule for LsEmailClientModule {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsEmailClientModule");
        Ok(())
    }
}
