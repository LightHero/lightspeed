pub mod config;
pub mod error;
pub mod model;
pub mod module;
pub mod service;
pub mod utils;

#[cfg(feature = "web")]
pub mod web;

use crate::error::LsError;
use crate::service::auth::InMemoryRolesProvider;
use log::info;
use std::sync::Arc;

#[derive(Clone)]
pub struct LsCoreModule {
    pub auth: Arc<service::auth::LsAuthService>,
    pub jwt: Arc<service::jwt::LsJwtService>,
}

impl LsCoreModule {
    pub fn new(config: config::CoreConfig) -> Result<LsCoreModule, LsError> {
        println!("Creating LsCoreModule");
        info!("Creating LsCoreModule");

        let jwt = Arc::new(service::jwt::LsJwtService::new(&config.jwt)?);
        let auth = Arc::new(service::auth::LsAuthService::new(InMemoryRolesProvider::new(vec![].into())));
        Ok(LsCoreModule { jwt, auth })
    }
}

impl module::LsModule for LsCoreModule {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsCoreModule");
        Ok(())
    }
}
