pub mod config;
pub mod error;
pub mod model;
pub mod module;
pub mod service;
pub mod utils;
pub mod web;

use crate::error::LightSpeedError;
use crate::service::auth::InMemoryRolesProvider;
use log::info;
use std::sync::Arc;

#[derive(Clone)]
pub struct CoreModule {
    pub auth: Arc<service::auth::AuthService<InMemoryRolesProvider>>,
    pub jwt: Arc<service::jwt::JwtService>,
}

impl CoreModule {
    pub fn new(config: config::CoreConfig) -> Result<CoreModule, LightSpeedError> {
        println!("Creating CoreModule");
        info!("Creating CoreModule");

        let jwt = Arc::new(service::jwt::JwtService::new(&config.jwt)?);
        let auth = Arc::new(service::auth::AuthService::new(InMemoryRolesProvider::new(
            vec![].into(),
        )));
        Ok(CoreModule { jwt, auth })
    }
}

#[async_trait::async_trait]
impl module::Module for CoreModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CoreModule");
        Ok(())
    }
}
