use crate::config::EmailClientConfig;
use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod model;
pub mod service;

#[derive(Clone)]
pub struct EmailClientModule {
    pub email_config: EmailClientConfig,
    pub email_client: Arc<Box<dyn service::email::EmailClient>>,
}

impl EmailClientModule {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {
        println!("Creating EmailClientModule");
        info!("Creating EmailClientModule");

        let email_service = Arc::new(service::email::new(email_config.clone())?);

        Ok(EmailClientModule {
            email_config,
            email_client: email_service,
        })
    }
}

#[async_trait::async_trait(?Send)]
impl lightspeed_core::module::Module for EmailClientModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting EmailClientModule");
        Ok(())
    }
}
