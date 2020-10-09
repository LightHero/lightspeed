use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::Arc;
use crate::service::validation_code_service::ValidationCodeService;
use crate::service::hash_service::HashService;
use lightspeed_core::CoreModule;

pub mod dto;
pub mod service;

#[derive(Clone)]
pub struct ValidationCodeModule {
    pub hash_service: Arc<HashService>,
    pub validation_code_service: Arc<ValidationCodeService>,
}

impl ValidationCodeModule {
    pub fn new(core_module: &CoreModule) -> Result<Self, LightSpeedError> {
        println!("Creating ValidationCodeModule");
        info!("Creating ValidationCodeModule");

        let hash_service = Arc::new(HashService::new());

        let validation_code_service = Arc::new(ValidationCodeService::new(
            hash_service.clone(),
            core_module.jwt.clone(),
        ));

        Ok(ValidationCodeModule { hash_service, validation_code_service })
    }
}

#[async_trait::async_trait]
impl lightspeed_core::module::Module for ValidationCodeModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting ValidationCodeModule");
        Ok(())
    }
}
