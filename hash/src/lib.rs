use crate::service::hash_service::HashService;
use crate::service::validation_code_service::ValidationCodeService;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::CoreModule;
use log::*;
use std::sync::Arc;

pub mod dto;
pub mod service;

#[derive(Clone)]
pub struct HashModule {
    pub hash_service: Arc<HashService>,
    pub validation_code_service: Arc<ValidationCodeService>,
}

impl HashModule {
    pub fn new(core_module: &CoreModule) -> Result<Self, LightSpeedError> {
        println!("Creating HashModule");
        info!("Creating HashModule");

        let hash_service = Arc::new(HashService::new());

        let validation_code_service =
            Arc::new(ValidationCodeService::new(hash_service.clone(), core_module.jwt.clone()));

        Ok(HashModule { hash_service, validation_code_service })
    }
}

#[async_trait::async_trait]
impl lightspeed_core::module::Module for HashModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting HashModule");
        Ok(())
    }
}
