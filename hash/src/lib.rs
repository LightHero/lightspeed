use crate::service::hash_service::LsHashService;
use crate::service::validation_code_service::LsValidationCodeService;
use lightspeed_core::error::LsError;
use lightspeed_core::LsCoreModule;
use log::*;
use std::sync::Arc;

pub mod dto;
pub mod service;

#[derive(Clone)]
pub struct LsHashModule {
    pub hash_service: Arc<LsHashService>,
    pub validation_code_service: Arc<LsValidationCodeService>,
}

impl LsHashModule {
    pub fn new(core_module: &LsCoreModule) -> Result<Self, LsError> {
        println!("Creating LsHashModule");
        info!("Creating LsHashModule");

        let hash_service = Arc::new(LsHashService::new());

        let validation_code_service =
            Arc::new(LsValidationCodeService::new(hash_service.clone(), core_module.jwt.clone()));

        Ok(LsHashModule { hash_service, validation_code_service })
    }
}

#[async_trait::async_trait]
impl lightspeed_core::module::LsModule for LsHashModule {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsHashModule");
        Ok(())
    }
}
