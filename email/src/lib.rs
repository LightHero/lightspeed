use crate::config::EmailClientConfig;
use crate::service::EmailService;
use lightspeed_core::error::LightSpeedError;
use log::*;

pub mod config;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct EmailClientModule {
    pub email_config: EmailClientConfig,
    pub email_service: EmailService,
}

impl EmailClientModule {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {
        println!("Creating EmailClientModule");
        info!("Creating EmailClientModule");

        let email_service = EmailService::new(repository::email::new(email_config.clone())?);

        Ok(EmailClientModule {
            email_config,
            email_service,
        })
    }
}

#[async_trait::async_trait]
impl lightspeed_core::module::Module for EmailClientModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting EmailClientModule");
        Ok(())
    }
}
